use bevy::{
    prelude::*,
    render::{
        renderer::RenderDevice,
        texture::TextureFormatPixelInfo,
    }
};

pub mod object_table;
pub mod camera_table;

pub use camera_table::CameraOutputTable;
pub use object_table::SegmentationDataTable;


use crate::utils::image_copy::*;

/// Setups image saver
pub struct InternalCameraOutput;
impl Plugin for InternalCameraOutput {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PostUpdate, (update_camera_table, save_camera_table_to_file).chain())
            .add_plugins(ImageCopyPlugin);
    }
}

// Takes from channel image content sent from render world and saves it to disk
fn update_camera_table(
    mut image_table: ResMut<CameraOutputTable>,
    mut images: ResMut<Assets<Image>>,
) {
    if image_table.preroll < 1 {
        // We don't want to block the main world on this,
        // so we use try_recv which attempts to receive without blocking
        for (image, receiver) in image_table.image_handles.iter().zip(image_table.receivers.iter()) {
            
            let mut image_data = Vec::new();
            
            while let Ok(data) = receiver.try_recv() {
                // image generation could be faster than saving to fs,
                // that's why use only last of them
                image_data = data;
            }

            if !image_data.is_empty() {
                // info!("Copying Image to table");
                // Fill correct data from channel to image
                let img_bytes = images.get_mut(image.id()).unwrap();

                // We need to ensure that this works regardless of the image dimensions
                // If the image became wider when copying from the texture to the buffer,
                // then the data is reduced to its original size when copying from the buffer to the image.
                let row_bytes = img_bytes.width() as usize
                    * img_bytes.texture_descriptor.format.pixel_size();
                let aligned_row_bytes = RenderDevice::align_copy_bytes_per_row(row_bytes);
                
                if row_bytes == aligned_row_bytes {
                    img_bytes.data.clone_from(&image_data);
                } else {
                    // shrink data to original image size
                    img_bytes.data = image_data
                        .chunks(aligned_row_bytes)
                        .take(img_bytes.height() as usize)
                        .flat_map(|row| &row[..row_bytes.min(row.len())])
                        .cloned()
                        .collect();
                }
            }
        }
    } else {
        // clears channel for skipped frames
        for receiver in image_table.receivers.iter() {
            while receiver.try_recv().is_ok() {}
        }
        image_table.preroll -= 1;
    }
}

fn save_camera_table_to_file(
    input: Res<ButtonInput<KeyCode>>,
    image_table: ResMut<CameraOutputTable>,
    mut images: ResMut<Assets<Image>>,
) {
    if input.just_pressed(KeyCode::KeyS) {
        info!("Saving Camera Table to files");
        image_table.save_images_to_file(&mut images);
    }
}