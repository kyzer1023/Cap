use super::H264Encoder;
use crate::{
    d3d::create_d3d_device,
    input_sample_pool::{
        DxgiInputSamples, allocate_dxgi_input_sample, media_type_frame_duration_hns,
    },
    media::MF_VERSION,
};
use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use windows::{
    Graphics::SizeInt32,
    Win32::{
        Foundation::FILETIME,
        Graphics::{
            Direct3D11::{
                D3D11_BIND_RENDER_TARGET, D3D11_BIND_SHADER_RESOURCE, D3D11_TEXTURE2D_DESC,
                D3D11_USAGE_DEFAULT, ID3D11Device, ID3D11Texture2D,
            },
            Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC},
        },
        Media::MediaFoundation::{
            self, IMFAttributes, IMFByteStream, IMFSample, MF_E_NO_EVENTS_AVAILABLE,
            MF_EVENT_FLAG_NO_WAIT, MF_EVENT_TYPE, MF_SOURCE_READER_FIRST_VIDEO_STREAM,
            MF_SOURCE_READERF_ENDOFSTREAM, MF_SOURCE_READERF_ERROR, MFCreateSinkWriterFromURL,
            MFCreateSourceReaderFromURL, MFSTARTUP_FULL, MFStartup, MFT_MESSAGE_COMMAND_DRAIN,
            MFT_MESSAGE_COMMAND_FLUSH, MFT_MESSAGE_NOTIFY_BEGIN_STREAMING,
            MFT_MESSAGE_NOTIFY_END_OF_STREAM, MFT_MESSAGE_NOTIFY_END_STREAMING,
            MFT_MESSAGE_NOTIFY_START_OF_STREAM, MFT_OUTPUT_DATA_BUFFER,
        },
        System::{
            Threading::{GetCurrentProcess, GetProcessTimes},
            WinRT::{RO_INIT_MULTITHREADED, RoInitialize},
        },
    },
    core::PCWSTR,
};

const WARMUP_FRAMES: usize = 30;
const TIMED_FRAMES: usize = 300;
const FRAME_RATE: u32 = 30;
const BITRATE_MULTIPLIER: f32 = 0.15;
const RUNS_PER_PATH: usize = 5;
const TRIAL_DEADLINE: Duration = Duration::from_secs(180);

struct Trial {
    process_input_ns: Vec<u64>,
    samples: Vec<IMFSample>,
    submitted: usize,
    wall: Duration,
    cpu_hns: u64,
}

pub fn run_mf_input_sample_bench() -> Result<(), String> {
    let paths = selected_paths()?;
    unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.map_err(|error| error.to_string())?;
    unsafe { MFStartup(MF_VERSION, MFSTARTUP_FULL) }.map_err(|error| error.to_string())?;

    let device = create_d3d_device().map_err(|error| error.to_string())?;
    println!(
        "mf-input-sample-bench fps={FRAME_RATE} warmup={WARMUP_FRAMES} timed={TIMED_FRAMES} runs_per_path={RUNS_PER_PATH} bitrate_multiplier={BITRATE_MULTIPLIER}"
    );

    for (width, height) in [(1920u32, 1080u32), (3840, 2160)] {
        let texture = solid_texture(&device, width, height)?;
        let mut summaries = Vec::new();
        for (index, reuse) in paths.iter().copied().enumerate() {
            let mut encoder = H264Encoder::new(
                &device,
                DXGI_FORMAT_B8G8R8A8_UNORM,
                SizeInt32 {
                    Width: width as i32,
                    Height: height as i32,
                },
                FRAME_RATE,
                BITRATE_MULTIPLIER,
            )
            .map_err(|error| error.to_string())?;
            encoder.validate().map_err(|error| error.to_string())?;

            let mut trial = encode_trial(&mut encoder, reuse, &texture)?;
            if trial.process_input_ns.len() != TIMED_FRAMES {
                return Err(format!(
                    "{width}x{height} {} run {index} timed {} process-input samples, expected {TIMED_FRAMES}",
                    path_name(reuse),
                    trial.process_input_ns.len(),
                ));
            }
            let (min_ns, median_ns, p95_ns, max_ns) = spread(&mut trial.process_input_ns);
            let path = path_name(reuse);
            let output = output_path(width, height, index, path)?;
            let written = write_mp4(&encoder, &trial.samples, &output)?;
            let decoded = decode_frame_count(&output)?;
            println!(
                "RESULT size={width}x{height} path={path} run={index} process_input_min_us={} process_input_median_us={} process_input_p95_us={} process_input_max_us={} wall_ms={:.3} cpu_ms={:.3} submitted={} timed={TIMED_FRAMES} outputs={written} decoded={decoded}",
                min_ns / 1_000,
                median_ns / 1_000,
                p95_ns / 1_000,
                max_ns / 1_000,
                trial.wall.as_secs_f64() * 1_000.0,
                trial.cpu_hns as f64 / 10_000.0,
                trial.submitted,
            );
            if trial.submitted != WARMUP_FRAMES + TIMED_FRAMES
                || written != trial.submitted
                || decoded != trial.submitted
            {
                return Err(format!(
                    "{width}x{height} {path} run {index} frame count mismatch: submitted={} timed_samples={} outputs={written} decoded={decoded} file={}",
                    trial.submitted,
                    trial.process_input_ns.len(),
                    output.display(),
                ));
            }
            std::fs::remove_file(&output).map_err(|error| error.to_string())?;
            summaries.push((reuse, median_ns));
        }
        print_summary(width, height, &summaries);
    }
    Ok(())
}

fn selected_paths() -> Result<Vec<bool>, String> {
    let mode = std::env::var("MF_INPUT_SAMPLE_PATH").unwrap_or_else(|_| "both".to_string());
    match mode.as_str() {
        "both" => {
            let mut paths = Vec::with_capacity(RUNS_PER_PATH * 2);
            for repeat in 0..RUNS_PER_PATH {
                if repeat % 2 == 0 {
                    paths.push(false);
                    paths.push(true);
                } else {
                    paths.push(true);
                    paths.push(false);
                }
            }
            Ok(paths)
        }
        "allocate" => Ok(vec![false; RUNS_PER_PATH]),
        "reuse" => Ok(vec![true; RUNS_PER_PATH]),
        other => Err(format!(
            "MF_INPUT_SAMPLE_PATH must be both, allocate, or reuse (got {other})"
        )),
    }
}

fn path_name(reuse: bool) -> &'static str {
    if reuse { "reuse" } else { "allocate" }
}

fn encode_trial(
    encoder: &mut H264Encoder,
    reuse: bool,
    texture: &ID3D11Texture2D,
) -> Result<Trial, String> {
    let duration =
        media_type_frame_duration_hns(encoder.output_type()).map_err(|error| error.to_string())?;
    let submitted_target = WARMUP_FRAMES + TIMED_FRAMES;
    let mut pool = DxgiInputSamples::new();
    let mut inputs = 0usize;
    let mut process_input_ns = Vec::with_capacity(TIMED_FRAMES);
    let mut samples = Vec::new();
    let mut draining = false;
    let mut wall_start = None;
    let mut cpu_start = None;
    let started = Instant::now();

    unsafe {
        encoder
            .transform
            .ProcessMessage(MFT_MESSAGE_COMMAND_FLUSH, 0)
            .map_err(|error| error.to_string())?;
        encoder
            .transform
            .ProcessMessage(MFT_MESSAGE_NOTIFY_BEGIN_STREAMING, 0)
            .map_err(|error| error.to_string())?;
        encoder
            .transform
            .ProcessMessage(MFT_MESSAGE_NOTIFY_START_OF_STREAM, 0)
            .map_err(|error| error.to_string())?;
    }

    loop {
        if started.elapsed() > TRIAL_DEADLINE {
            return Err(format!(
                "encode trial exceeded {}s after {inputs} inputs and {} outputs",
                TRIAL_DEADLINE.as_secs(),
                samples.len()
            ));
        }
        let event = match pull_event(encoder)? {
            Some(event) => event,
            None => {
                std::thread::sleep(Duration::from_millis(1));
                continue;
            }
        };
        let event_type =
            MF_EVENT_TYPE(unsafe { event.GetType() }.map_err(|error| error.to_string())? as i32);
        match event_type {
            MediaFoundation::METransformNeedInput => {
                if inputs == submitted_target {
                    if !draining {
                        unsafe {
                            encoder
                                .transform
                                .ProcessMessage(MFT_MESSAGE_NOTIFY_END_OF_STREAM, 0)
                                .map_err(|error| error.to_string())?;
                            encoder
                                .transform
                                .ProcessMessage(MFT_MESSAGE_COMMAND_DRAIN, 0)
                                .map_err(|error| error.to_string())?;
                        }
                        draining = true;
                    }
                    continue;
                }
                let time = inputs as i64 * duration;
                if inputs == WARMUP_FRAMES {
                    wall_start = Some(Instant::now());
                    cpu_start = Some(process_cpu_hns()?);
                }
                encoder
                    .video_processor
                    .process_texture(texture)
                    .map_err(|error| error.to_string())?;
                let section = Instant::now();
                let sample = {
                    let output = encoder.video_processor.output_texture();
                    if reuse {
                        pool.sample_for(output, time, duration)
                    } else {
                        allocate_dxgi_input_sample(output, time, duration)
                    }
                }
                .map_err(|error| error.to_string())?;
                unsafe {
                    encoder
                        .transform
                        .ProcessInput(encoder.input_stream_id, &sample, 0)
                        .map_err(|error| error.to_string())?;
                }
                if inputs >= WARMUP_FRAMES {
                    process_input_ns.push(nanos(section.elapsed()));
                }
                inputs += 1;
            }
            MediaFoundation::METransformHaveOutput => {
                let mut status = 0;
                let mut output_buffers = [MFT_OUTPUT_DATA_BUFFER {
                    dwStreamID: encoder.output_stream_id,
                    ..Default::default()
                }];
                unsafe {
                    encoder
                        .transform
                        .ProcessOutput(0, &mut output_buffers, &mut status)
                        .map_err(|error| error.to_string())?;
                }
                if let Some(sample) = output_buffers[0].pSample.take() {
                    samples.push(sample);
                }
            }
            MediaFoundation::METransformDrainComplete if draining => break,
            other => {
                return Err(format!("unexpected media event {}", other.0));
            }
        }
    }

    unsafe {
        encoder
            .transform
            .ProcessMessage(MFT_MESSAGE_NOTIFY_END_STREAMING, 0)
            .map_err(|error| error.to_string())?;
        encoder
            .transform
            .ProcessMessage(MFT_MESSAGE_COMMAND_FLUSH, 0)
            .map_err(|error| error.to_string())?;
    }

    let wall = wall_start
        .ok_or_else(|| "timed window never started".to_string())?
        .elapsed();
    let cpu_hns = process_cpu_hns()?
        .saturating_sub(cpu_start.ok_or_else(|| "timed window never started".to_string())?);
    Ok(Trial {
        process_input_ns,
        samples,
        submitted: inputs,
        wall,
        cpu_hns,
    })
}

fn pull_event(
    encoder: &H264Encoder,
) -> Result<Option<windows::Win32::Media::MediaFoundation::IMFMediaEvent>, String> {
    match unsafe { encoder.event_generator.GetEvent(MF_EVENT_FLAG_NO_WAIT) } {
        Ok(event) => Ok(Some(event)),
        Err(error) if error.code() == MF_E_NO_EVENTS_AVAILABLE => Ok(None),
        Err(error) => Err(error.to_string()),
    }
}

fn write_mp4(encoder: &H264Encoder, samples: &[IMFSample], path: &Path) -> Result<usize, String> {
    let wide = wide_path(path);
    let writer = unsafe {
        MFCreateSinkWriterFromURL(
            PCWSTR(wide.as_ptr()),
            None::<&IMFByteStream>,
            None::<&IMFAttributes>,
        )
    }
    .map_err(|error| format!("MFCreateSinkWriterFromURL: {error}"))?;
    let stream = unsafe { writer.AddStream(encoder.output_type()) }
        .map_err(|error| format!("AddStream: {error}"))?;
    unsafe { writer.SetInputMediaType(stream, encoder.output_type(), None::<&IMFAttributes>) }
        .map_err(|error| format!("SetInputMediaType: {error}"))?;
    unsafe { writer.BeginWriting() }.map_err(|error| format!("BeginWriting: {error}"))?;
    for sample in samples {
        unsafe { writer.WriteSample(stream, sample) }
            .map_err(|error| format!("WriteSample: {error}"))?;
    }
    unsafe { writer.Finalize() }.map_err(|error| format!("Finalize: {error}"))?;
    Ok(samples.len())
}

fn decode_frame_count(path: &Path) -> Result<usize, String> {
    let wide = wide_path(path);
    let reader =
        unsafe { MFCreateSourceReaderFromURL(PCWSTR(wide.as_ptr()), None::<&IMFAttributes>) }
            .map_err(|error| format!("MFCreateSourceReaderFromURL: {error}"))?;
    let mut decoded = 0usize;
    let end_flag = MF_SOURCE_READERF_ENDOFSTREAM.0 as u32;
    let error_flag = MF_SOURCE_READERF_ERROR.0 as u32;
    let read_cap = WARMUP_FRAMES + TIMED_FRAMES + 2;
    for _ in 0..read_cap {
        let mut flags = 0u32;
        let mut sample: Option<IMFSample> = None;
        unsafe {
            reader.ReadSample(
                MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32,
                0,
                None,
                Some(std::ptr::addr_of_mut!(flags)),
                None,
                Some(std::ptr::addr_of_mut!(sample)),
            )
        }
        .map_err(|error| format!("ReadSample: {error}"))?;
        if flags & error_flag != 0 {
            return Err(format!("source reader error flag {flags}"));
        }
        if sample.is_some() {
            decoded += 1;
        }
        if flags & end_flag != 0 {
            return Ok(decoded);
        }
    }
    Err(format!(
        "decoder did not reach end of stream after {decoded} frames in {read_cap} reads"
    ))
}

fn solid_texture(
    device: &ID3D11Device,
    width: u32,
    height: u32,
) -> Result<ID3D11Texture2D, String> {
    let desc = D3D11_TEXTURE2D_DESC {
        Width: width,
        Height: height,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            ..Default::default()
        },
        Usage: D3D11_USAGE_DEFAULT,
        BindFlags: (D3D11_BIND_RENDER_TARGET.0 | D3D11_BIND_SHADER_RESOURCE.0) as u32,
        ..Default::default()
    };
    let mut texture = None;
    unsafe { device.CreateTexture2D(&desc, None, Some(&mut texture)) }
        .map_err(|error| error.to_string())?;
    texture.ok_or_else(|| "CreateTexture2D returned no texture".to_string())
}

fn process_cpu_hns() -> Result<u64, String> {
    let mut creation = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    unsafe {
        GetProcessTimes(
            GetCurrentProcess(),
            &mut creation,
            &mut exit,
            &mut kernel,
            &mut user,
        )
    }
    .map_err(|error| error.to_string())?;
    Ok(filetime_hns(kernel).saturating_add(filetime_hns(user)))
}

fn filetime_hns(time: FILETIME) -> u64 {
    (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime)
}

fn wide_path(path: &Path) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    path.as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

fn output_path(width: u32, height: u32, index: usize, path: &str) -> Result<PathBuf, String> {
    let directory = std::env::temp_dir().join("mf-input-sample-bench");
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(directory.join(format!("{width}x{height}-{index}-{path}.mp4")))
}

fn spread(samples: &mut [u64]) -> (u64, u64, u64, u64) {
    samples.sort_unstable();
    let median = percentile(samples, 1, 2);
    let p95 = percentile(samples, 95, 100);
    (samples[0], median, p95, samples[samples.len() - 1])
}

fn percentile(sorted: &[u64], numerator: usize, denominator: usize) -> u64 {
    let index = (sorted.len() * numerator).div_ceil(denominator) - 1;
    sorted[index]
}

fn print_summary(width: u32, height: u32, runs: &[(bool, u64)]) {
    for reuse in [false, true] {
        let medians: Vec<u64> = runs
            .iter()
            .filter(|(path, _)| *path == reuse)
            .map(|(_, median)| *median)
            .collect();
        if medians.is_empty() {
            continue;
        }
        let mut ordered = medians;
        ordered.sort_unstable();
        let mid = percentile(&ordered, 1, 2);
        println!(
            "SUMMARY size={width}x{height} path={} runs={} median_of_medians_us={} min_median_us={} max_median_us={}",
            path_name(reuse),
            ordered.len(),
            mid / 1_000,
            ordered[0] / 1_000,
            ordered[ordered.len() - 1] / 1_000,
        );
    }
}

fn nanos(duration: Duration) -> u64 {
    u64::try_from(duration.as_nanos()).unwrap_or(u64::MAX)
}
