#[cfg(windows)]
pub mod d3d;
#[cfg(windows)]
pub mod media;
#[cfg(windows)]
pub mod mft;
#[cfg(windows)]
pub mod video;

#[cfg(any(windows, test))]
pub(crate) mod input_sample_pool;

#[cfg(windows)]
pub use video::H264Encoder;
