const MAX_TRACKED_SAMPLES: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureKey(u64);

impl TextureKey {
    pub const fn new(identity: u64) -> Self {
        Self(identity)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SampleLease {
    slot: u8,
    generation: u64,
}

impl SampleLease {
    pub const fn slot(self) -> usize {
        self.slot as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SamplePlan {
    Reuse(SampleLease),
    Create(SampleLease),
    OneShot,
}

#[derive(Clone, Copy)]
struct Slot {
    texture: Option<TextureKey>,
    generation: Option<u64>,
}

pub struct InputSamplePool {
    slots: [Slot; MAX_TRACKED_SAMPLES],
    next_generation: u64,
}

impl InputSamplePool {
    pub const fn new() -> Self {
        Self {
            slots: [Slot {
                texture: None,
                generation: None,
            }; MAX_TRACKED_SAMPLES],
            next_generation: 1,
        }
    }

    pub fn acquire(&mut self, texture: TextureKey) -> SamplePlan {
        if let Some(index) = self
            .slots
            .iter()
            .position(|slot| slot.texture == Some(texture) && slot.generation.is_none())
        {
            return SamplePlan::Reuse(self.checkout(index, texture));
        }
        if let Some(index) = self.slots.iter().position(|slot| slot.generation.is_none()) {
            return SamplePlan::Create(self.checkout(index, texture));
        }
        SamplePlan::OneShot
    }

    pub fn release(&mut self, lease: SampleLease) -> bool {
        self.finish(lease, true)
    }

    pub fn abandon(&mut self, lease: SampleLease) -> bool {
        self.finish(lease, false)
    }

    fn checkout(&mut self, index: usize, texture: TextureKey) -> SampleLease {
        let generation = self.next_generation;
        self.next_generation += 1;
        self.slots[index] = Slot {
            texture: Some(texture),
            generation: Some(generation),
        };
        SampleLease {
            slot: index as u8,
            generation,
        }
    }

    fn finish(&mut self, lease: SampleLease, keep_texture: bool) -> bool {
        let Some(slot) = self.slots.get_mut(lease.slot()) else {
            return false;
        };
        if slot.generation != Some(lease.generation) {
            return false;
        }
        slot.generation = None;
        if !keep_texture {
            slot.texture = None;
        }
        true
    }
}

#[cfg(windows)]
mod dxgi {
    use super::{InputSamplePool, MAX_TRACKED_SAMPLES, SampleLease, SamplePlan, TextureKey};
    use windows::{
        Win32::{
            Foundation::E_INVALIDARG,
            Graphics::Direct3D11::ID3D11Texture2D,
            Media::MediaFoundation::{
                IMFMediaBuffer, IMFMediaType, IMFSample, MF_MT_FRAME_RATE,
                MFCreateDXGISurfaceBuffer, MFCreateSample,
            },
        },
        core::{Interface, Result},
    };

    pub(crate) fn media_type_frame_duration_hns(media_type: &IMFMediaType) -> Result<i64> {
        let packed = unsafe { media_type.GetUINT64(&MF_MT_FRAME_RATE)? };
        let numerator = (packed >> 32) as u32;
        let denominator = packed as u32;
        if numerator == 0 {
            return Err(windows::core::Error::new(
                E_INVALIDARG,
                "MF_MT_FRAME_RATE numerator is 0",
            ));
        }
        Ok(10_000_000i64 * i64::from(denominator) / i64::from(numerator))
    }

    struct TrackedSample {
        sample: IMFSample,
        buffer: IMFMediaBuffer,
        lease: SampleLease,
    }

    pub(crate) struct DxgiInputSamples {
        pool: InputSamplePool,
        slots: [Option<TrackedSample>; MAX_TRACKED_SAMPLES],
    }

    impl DxgiInputSamples {
        pub(crate) fn new() -> Self {
            Self {
                pool: InputSamplePool::new(),
                slots: std::array::from_fn(|_| None),
            }
        }

        pub(crate) fn sample_for(
            &mut self,
            texture: &ID3D11Texture2D,
            time_hns: i64,
            duration_hns: i64,
        ) -> Result<IMFSample> {
            self.observe_releases();
            let key = TextureKey::new(Interface::as_raw(texture) as usize as u64);
            match self.pool.acquire(key) {
                SamplePlan::OneShot => {
                    fresh_sample(texture, time_hns, duration_hns).map(|(sample, _buffer)| sample)
                }
                SamplePlan::Reuse(lease) => self.reuse(lease, texture, time_hns, duration_hns),
                SamplePlan::Create(lease) => self.create(lease, texture, time_hns, duration_hns),
            }
        }

        fn observe_releases(&mut self) {
            let pool = &mut self.pool;
            let slots = &self.slots;
            for tracked in slots.iter().flatten() {
                if sole_pool_reference(&tracked.sample) {
                    pool.release(tracked.lease);
                }
            }
        }

        fn reuse(
            &mut self,
            lease: SampleLease,
            texture: &ID3D11Texture2D,
            time_hns: i64,
            duration_hns: i64,
        ) -> Result<IMFSample> {
            let Some(tracked) = self.slots[lease.slot()].as_mut() else {
                return self.create(lease, texture, time_hns, duration_hns);
            };
            unsafe {
                tracked.sample.RemoveAllBuffers()?;
                tracked.sample.AddBuffer(&tracked.buffer)?;
                tracked.sample.SetSampleTime(time_hns)?;
                tracked.sample.SetSampleDuration(duration_hns)?;
            }
            tracked.lease = lease;
            Ok(tracked.sample.clone())
        }

        fn create(
            &mut self,
            lease: SampleLease,
            texture: &ID3D11Texture2D,
            time_hns: i64,
            duration_hns: i64,
        ) -> Result<IMFSample> {
            let (sample, buffer) = match fresh_sample(texture, time_hns, duration_hns) {
                Ok(built) => built,
                Err(error) => {
                    self.pool.abandon(lease);
                    return Err(error);
                }
            };
            let cloned = sample.clone();
            self.slots[lease.slot()] = Some(TrackedSample {
                sample,
                buffer,
                lease,
            });
            Ok(cloned)
        }
    }

    pub(crate) fn allocate_dxgi_input_sample(
        texture: &ID3D11Texture2D,
        time_hns: i64,
        duration_hns: i64,
    ) -> Result<IMFSample> {
        fresh_sample(texture, time_hns, duration_hns).map(|(sample, _buffer)| sample)
    }

    fn fresh_sample(
        texture: &ID3D11Texture2D,
        time_hns: i64,
        duration_hns: i64,
    ) -> Result<(IMFSample, IMFMediaBuffer)> {
        unsafe {
            let buffer = MFCreateDXGISurfaceBuffer(&ID3D11Texture2D::IID, texture, 0, false)?;
            let sample = MFCreateSample()?;
            sample.AddBuffer(&buffer)?;
            sample.SetSampleTime(time_hns)?;
            sample.SetSampleDuration(duration_hns)?;
            Ok((sample, buffer))
        }
    }

    #[repr(C)]
    struct UnknownVtbl {
        _query_interface: unsafe extern "system" fn(
            *mut core::ffi::c_void,
            *const windows::core::GUID,
            *mut *mut core::ffi::c_void,
        ) -> windows::core::HRESULT,
        add_ref: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
        release: unsafe extern "system" fn(*mut core::ffi::c_void) -> u32,
    }

    const POOL_REF_PLUS_PROBE: u32 = 2;
    const POOL_REF_ONLY: u32 = 1;

    fn sole_pool_reference(sample: &IMFSample) -> bool {
        unsafe {
            let raw = Interface::as_raw(sample);
            let vtable = *(raw as *const *const UnknownVtbl);
            let after_add = ((*vtable).add_ref)(raw);
            let after_release = ((*vtable).release)(raw);
            after_add == POOL_REF_PLUS_PROBE && after_release == POOL_REF_ONLY
        }
    }
}

#[cfg(windows)]
pub(crate) use dxgi::{
    DxgiInputSamples, allocate_dxgi_input_sample, media_type_frame_duration_hns,
};

#[cfg(test)]
mod tests {
    use super::{InputSamplePool, SamplePlan, TextureKey};

    fn key(identity: u64) -> TextureKey {
        TextureKey::new(identity)
    }

    fn slot(plan: SamplePlan) -> usize {
        match plan {
            SamplePlan::Reuse(lease) | SamplePlan::Create(lease) => lease.slot(),
            SamplePlan::OneShot => panic!("expected a tracked slot"),
        }
    }

    #[test]
    fn first_checkout_creates_a_slot_and_a_later_release_reuses_it() {
        let mut pool = InputSamplePool::new();
        let created = pool.acquire(key(1));
        assert!(matches!(created, SamplePlan::Create(_)));
        assert_eq!(slot(created), 0);
        let SamplePlan::Create(lease) = created else {
            return;
        };
        assert!(pool.release(lease));

        let reused = pool.acquire(key(1));
        assert!(matches!(reused, SamplePlan::Reuse(_)));
        assert_eq!(slot(reused), 0);
    }

    #[test]
    fn a_held_sample_is_never_handed_out_again() {
        let mut pool = InputSamplePool::new();
        let first = pool.acquire(key(7));
        let second = pool.acquire(key(7));
        let third = pool.acquire(key(7));
        assert!(matches!(first, SamplePlan::Create(_)));
        assert!(matches!(second, SamplePlan::Create(_)));
        assert!(matches!(third, SamplePlan::Create(_)));
        assert_eq!([slot(first), slot(second), slot(third)], [0, 1, 2]);

        assert_eq!(pool.acquire(key(7)), SamplePlan::OneShot);

        let SamplePlan::Create(first_lease) = first else {
            return;
        };
        assert!(pool.release(first_lease));
        let reused = pool.acquire(key(7));
        assert!(matches!(reused, SamplePlan::Reuse(_)));
        assert_eq!(slot(reused), 0);
    }

    #[test]
    fn a_stale_release_does_not_free_the_newer_lease() {
        let mut pool = InputSamplePool::new();
        let SamplePlan::Create(first) = pool.acquire(key(3)) else {
            panic!("first checkout creates");
        };
        assert!(pool.release(first));
        let SamplePlan::Reuse(second) = pool.acquire(key(3)) else {
            panic!("released slot is reused");
        };

        assert!(!pool.release(first));
        let while_second_is_held = pool.acquire(key(3));
        assert!(matches!(while_second_is_held, SamplePlan::Create(_)));
        assert_ne!(slot(while_second_is_held), second.slot());

        assert!(pool.release(second));
        let reused = pool.acquire(key(3));
        assert!(matches!(reused, SamplePlan::Reuse(_)));
        assert_eq!(slot(reused), second.slot());
    }

    #[test]
    fn a_free_slot_for_another_texture_is_recreated() {
        let mut pool = InputSamplePool::new();
        let SamplePlan::Create(lease) = pool.acquire(key(1)) else {
            panic!("first checkout creates");
        };
        assert!(pool.release(lease));

        let changed = pool.acquire(key(2));
        assert!(matches!(changed, SamplePlan::Create(_)));
        assert_eq!(slot(changed), 0);
    }

    #[test]
    fn a_held_slot_stays_out_when_the_texture_changes() {
        let mut pool = InputSamplePool::new();
        let SamplePlan::Create(held) = pool.acquire(key(1)) else {
            panic!("first checkout creates");
        };
        let changed = pool.acquire(key(2));
        assert!(matches!(changed, SamplePlan::Create(_)));
        assert_eq!(slot(changed), 1);

        assert!(pool.release(held));
        let SamplePlan::Create(other) = changed else {
            return;
        };
        assert!(pool.release(other));

        let reused = pool.acquire(key(2));
        assert!(matches!(reused, SamplePlan::Reuse(_)));
        assert_eq!(slot(reused), 1);

        let replaced = pool.acquire(key(2));
        assert!(matches!(replaced, SamplePlan::Create(_)));
        assert_eq!(slot(replaced), 0);
    }

    #[test]
    fn an_abandoned_slot_is_created_again_for_the_same_texture() {
        let mut pool = InputSamplePool::new();
        let SamplePlan::Create(lease) = pool.acquire(key(4)) else {
            panic!("first checkout creates");
        };
        assert!(pool.abandon(lease));
        assert!(!pool.release(lease));

        let again = pool.acquire(key(4));
        assert!(matches!(again, SamplePlan::Create(_)));
        assert_eq!(slot(again), 0);
    }
}
