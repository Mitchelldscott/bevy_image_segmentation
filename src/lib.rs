// Define Modules
pub mod components;
pub mod plugin;
pub mod resources;
pub mod utils;

// Re-export user interface types
pub use components::{SegmentationObject, SegmentationCamera, RGBCamera};
// pub use camera::SegmentationCameraBundle;

pub use plugin::SegmentationPlugin;
