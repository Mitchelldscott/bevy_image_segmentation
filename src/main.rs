
use std::collections::HashMap;

use bevy::{
    prelude::*,
    render::view::RenderLayers,
};

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct SegmentationCamera;

#[derive(Component)]
struct SegmentationChildSpawned;

#[derive(Component)]
struct SegmentationObject {
    name: String,
}

// #[derive(Debug)]
// struct SegmentationInfo {
//     id: u32,
//     color: Color,
// }

// Define a resource to store all segmentation objects
#[derive(Resource, Default)]
struct SegmentationDataTable {
    table: HashMap<String, (u32, Color)>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, spawn_segmentation_children)
        .add_systems(Update, toggle_segmentation_display.run_if(resource_changed::<ButtonInput<KeyCode>>))
        .run();
}

/// Set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Plane
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Plane3d::default().mesh().size(5.0, 5.0)),
            material: materials.add(Color::srgb(0.3, 0.5, 0.3)),
            ..default()
        },
        SegmentationObject { 
            name: String::from("ground")
        }
    ));

    // Cube
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Cuboid::default()),
            material: materials.add(Color::srgb(0.8, 0.7, 0.6)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        SegmentationObject { 
            name: String::from("cube")
        }
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
        RenderLayers::layer(0)
    ));

    // Camera
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera,
        RenderLayers::layer(0)
    ))
    .with_children(|parent| {
        parent.spawn((
            Camera3dBundle {
                camera: Camera {
                    // renders after / on top of the main camera
                    order: 1,
                    is_active: false,
                    clear_color: ClearColorConfig::None,
                    ..default()
                },
                ..default()
            },
            SegmentationCamera,
            RenderLayers::layer(1)
        ));
    });

    commands.init_resource::<SegmentationDataTable>();

}

fn random_color() -> Color {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let r: f32 = rng.gen(); // Generates a random float between 0.0 and 1.0
    let g: f32 = rng.gen();
    let b: f32 = rng.gen();

    Color::srgb(r, g, b)
}

fn spawn_segmentation_children(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut object_table: ResMut<SegmentationDataTable>,
    query: Query<(Entity, &Handle<Mesh>, &SegmentationObject), (With<SegmentationObject>, Added<SegmentationObject>, Without<SegmentationChildSpawned>)>,
) {
    for (entity, mesh_handle, segmentation_object) in query.iter() {
        // Create a child entity with the same mesh but a new material.
        // The color of material is not very relevant and only used for
        // visual effect. However, the color should be representative of
        // the objects class, which also corresponds to a specific value 
        // that is used in the image mask when a sample is saved.
        // sample = (image, mask, annotations) 
        let table_len = object_table.table.len() as u32;
        let (_, color) = object_table.table.entry(segmentation_object.name.clone()).or_insert((table_len, random_color()));
        
        commands.entity(entity)
        .with_children(|parent| {
            parent.spawn((
                PbrBundle {
                    mesh: mesh_handle.clone(),
                    material: materials.add(StandardMaterial {
                        unlit: true,
                        metallic: 0.0,
                        perceptual_roughness: 1.0,
                        base_color: *color, // Set your desired color
                        ..default()
                    }),
                    ..default()
                },
                RenderLayers::layer(1)
            ));
        });

        commands.entity(entity).insert((RenderLayers::layer(0), SegmentationChildSpawned));
    }
}

fn toggle_segmentation_display(
    keys: Res<ButtonInput<KeyCode>>,
    mut segmentation_camera: Query<&mut Camera, With<SegmentationCamera>>,
) {
    if keys.just_pressed(KeyCode::Space) || keys.just_released(KeyCode::Space) {
        let mut camera = segmentation_camera.single_mut();
        camera.is_active = !camera.is_active;
    }
}