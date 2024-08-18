use bevy::{
    prelude::Deref,
    ecs::component::Component   
};

#[derive(Component)]
pub struct SegmentationObjectParent;

#[derive(Component, Deref, PartialEq, Eq, Hash)]
pub struct SegmentationObject(pub String);

impl From<&str> for SegmentationObject {
    fn from(label: &str) -> Self {
        SegmentationObject(label.to_string())
    }
}

#[derive(Component, Default, Deref)]
pub struct RGBCamera(pub CameraDescription);

impl RGBCamera {
    pub fn new(name: &str, width: u32, height: u32) -> Self {
        RGBCamera(
            CameraDescription {
                name: name.to_string(),
                width,
                height
            }
        )
    }
}

#[derive(Component, Default, Deref)]
pub struct SegmentationCamera(pub CameraDescription);

#[derive(Clone)]
pub struct CameraDescription {
    pub name: String,
    pub width: u32, 
    pub height: u32, 
}

impl Default for CameraDescription {
    fn default() -> Self {
        CameraDescription {
            name: String::from("Camera0"),
            // directory: String::from("segmentation_dataset"),
            width: 512, 
            height: 512, 
        }
    }
}