use bevy::{
    prelude::*,
    asset::Handle,
    ecs::prelude::Resource,
    render::{
        camera::RenderTarget,
        render_asset::RenderAssetUsages,
        render_resource::{
            Extent3d, TextureDimension, TextureFormat,
            TextureUsages,
        },
        renderer::RenderDevice,
        texture::BevyDefault,
    }
};
use std::path::PathBuf;
use crossbeam_channel::{Receiver, Sender};
use crate::utils::image_copy::ImageCopier;

// CPU world resource to access images
#[derive(Resource)]
pub struct CameraOutputTable{
    pub camera_names: Vec<String>,
    pub image_handles: Vec<Handle<Image>>,
    pub preroll: u32,
    pub receivers: Vec<Receiver<Vec<u8>>>,
}

impl Default for CameraOutputTable {
    fn default() -> Self {
        CameraOutputTable {
            camera_names: vec![],
            image_handles: vec![],
            preroll: 40,
            receivers: vec![],
        }
    }
}

impl CameraOutputTable {
    pub fn link_new_target(&mut self, image_handle: Handle<Image>, camera_name: String) -> Sender<Vec<u8>> {
        
        let (s, r) = crossbeam_channel::unbounded();
        
        match self.camera_names.iter().position(|name| *name == camera_name) {
            Some(index) => {
                self.image_handles[index] = image_handle;
                self.receivers[index] = r;
            },
            None => {
                self.camera_names.push(camera_name);
                self.image_handles.push(image_handle);
                self.receivers.push(r);
            },
        }

        s
    }

    /// Setups render target and cpu image for saving, changes scene state into render mode
    pub fn create_render_target(
        &mut self,
        camera_name: String,
        width: u32,
        height: u32,
        commands: &mut Commands,
        images: &mut ResMut<Assets<Image>>,
        render_device: &Res<RenderDevice>,
    ) -> RenderTarget {

        let size = Extent3d {
            width: width,
            height: height,
            ..Default::default()
        };

        // This is the texture that will be rendered to.
        let mut render_target_image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0; 4],
            TextureFormat::bevy_default(),
            RenderAssetUsages::default(),
        );
        render_target_image.texture_descriptor.usage |=
            TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING;
        let render_target_image_handle = images.add(render_target_image);

        // This is the texture that will be copied to.
        let cpu_image = Image::new_fill(
            size,
            TextureDimension::D2,
            &[0; 4],
            TextureFormat::bevy_default(),
            RenderAssetUsages::default(),
        );
        let cpu_image_handle = images.add(cpu_image);

        let sender = self.link_new_target(cpu_image_handle, camera_name);

        commands.spawn(ImageCopier::new(
            sender,
            render_target_image_handle.clone(),
            size,
            render_device,
        ));

        RenderTarget::Image(render_target_image_handle)
    }

    pub fn save_images_to_file(&self, images: &mut ResMut<Assets<Image>>) {

        for (image, name) in self.image_handles.iter().zip(self.camera_names.iter()){

            let img_bytes = images.get_mut(image.id()).unwrap();

            // Create RGBA Image Buffer. Only for saving to file
            let img = match img_bytes.clone().try_into_dynamic() {
                Ok(img) => img.to_rgba8(),
                Err(e) => panic!("Failed to create image buffer {e:?}"),
            };
    
            // Prepare directory for images, test_images in bevy folder is used here for example
            // You should choose the path depending on your needs
            let images_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_images");
            std::fs::create_dir_all(&images_dir).unwrap();
    
            // Choose filename starting from 000.png
            let image_path = images_dir.join(format!("{:?}.png", name.clone()));
    
            // Finally saving image to file, this heavy blocking operation is kept here
            // for example simplicity, but in real app you should move it to a separate task
            if let Err(e) = img.save(image_path) {
                panic!("Failed to save image: {}", e);
            };
        }
    }
}