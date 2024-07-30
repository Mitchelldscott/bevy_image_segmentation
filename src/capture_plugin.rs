
use image::{ImageBuffer, Rgba};
use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};


#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct SegmentedCamera;


pub struct CapturePlugin;

impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        app
        .add_systems(Startup, setup)
        .add_systems(Update, capture_image_on_spacebar);
    }
}
// , asset_server: Res<AssetServer>, mut materials: ResMut<Assets<ColorMaterial>>
fn setup(mut commands: Commands, mut segmented_image: ResMut<Assets<Image>>) {
    
    // Image size
    let size = Extent3d {
        width: 512,
        height: 512,
        ..default()
    };

    // This is the texture that segmented images will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };

    // initialize image.data with zeroes
    image.resize(size);

    let segmented_image_handle = segmented_image.add(image);

    let camera_transform = Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y);

    commands.spawn((
        Camera3dBundle {
            transform: camera_transform,
            ..default()
        },
        MainCamera
    ));
    
    commands.spawn((
        Camera3dBundle {
            camera: Camera {
                order: -1,
                target: segmented_image_handle.clone().into(),
                clear_color: Color::WHITE.into(),
                ..default()
            },
            transform: camera_transform,
            ..default()
        },
        SegmentedCamera,
    ));
}

fn capture_image_on_spacebar(
    keys: Res<ButtonInput<KeyCode>>,
    segmented_image_handle: Res<Assets<Image>>
) {
    if keys.just_pressed(KeyCode::Space) {
        // Get the camera and image handle (assuming a single camera here)
        
        // Render to an offscreen texture
        // let width = 512;
        // let height = 512;
        // let mut texture = Image::new(
        //     Extent3d {
        //         width,
        //         height,
        //         depth_or_array_layers: 1,
        //     },
        //     TextureFormat::Rgba8Unorm,
        // );
        // if let Some(image) = segmented_image_handle.get(image) {

        // }

        // Save the texture as an image file
        let file_path = "output_image.png";
        save_image(file_path, &segmented_image_handle.data, 512, 512);
        println!("Image saved to {}", file_path);
    }
}

fn save_image(file_path: &str, data: &[u8], width: u32, height: u32) {
    let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, data.to_vec()).unwrap();
    img.save(file_path).unwrap();
}