//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy_image_segmentation::{
    SegmentationPlugin, 
    SegmentationObject,
    RGBCamera,
};

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SegmentationPlugin))
        .add_systems(Startup, setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    }, SegmentationObject::from("ground")));
    // cube
    commands.spawn((PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::srgb_u8(124, 144, 255)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    }, SegmentationObject::from("cuboid")));
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera with RGBCamera marker, a SegmentationCamera will be attached to so it can mimic the
    // view, camera_description and render target
    commands.spawn((Camera3dBundle{
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }, RGBCamera::default()));
}
