
use bevy::{
    prelude::*,
    render::{
        // camera::RenderTarget,
        // render_resource::{
        //     Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        // },
        view::RenderLayers,
    },
};

use bevy_image_segmentation::{SegmentationPlugin, SegmentationObject};


fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SegmentationPlugin))
        .add_systems(Startup, setup)
        .run();

}

/// Set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(5.0, 5.0)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            ..default()
        },
        SegmentationObject (String::from("ground")),
    ));

    // Cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        SegmentationObject (String::from("cube")),
    ));

    // Light
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        },
        RenderLayers::layer(0),
    ));

}
