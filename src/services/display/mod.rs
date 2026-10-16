// Display backends facade

pub mod drm_kms;
pub mod factory;
pub mod framebuffer;
pub mod simulator;
pub mod traits;

pub use factory::{BackendType, create_backend};
