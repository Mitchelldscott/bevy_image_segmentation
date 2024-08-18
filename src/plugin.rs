
use bevy::{
    // app::ScheduleRunnerPlugin,
    prelude::*,
    render::{
        camera::RenderTarget,
        renderer::RenderDevice,
        view::RenderLayers,
    },
};

// use std::time::Duration;
use crate::{
    components::*,
    resources::*,
};

pub struct SegmentationPlugin;

impl Plugin for SegmentationPlugin {
    fn build(&self, app: &mut App) {

        // Confirm required resources are initialized, if not loads the defaults
        app
            .init_resource::<SegmentationDataTable>()
            .init_resource::<CameraOutputTable>()
            // .insert_resource(ClearColor(Color::srgb_u8(0, 0, 0)))
            .add_systems(PostStartup, (spawn_segmentation_cameras, spawn_segmentation_materials))
            .add_systems(
                Update,
                toggle_segmentation_view.run_if(resource_changed::<ButtonInput<KeyCode>>),
            )
            // headless frame capture
            .add_plugins(InternalCameraOutput);
            // .add_plugins(ScheduleRunnerPlugin::run_loop(
            //     // Run 60 times per second.
            //     Duration::from_secs_f64(1.0 / 60.0),
            // ));
    }
}

fn spawn_segmentation_cameras(
    camera_query: Query<(Entity, &Camera, &RGBCamera)>,
    mut commands: Commands,
    mut image_table: ResMut<CameraOutputTable>,
    render_device: Res<RenderDevice>,
    mut images: ResMut<Assets<Image>>,
) {

    for (entity, camera, camera_description) in camera_query.iter() {

        let mut segmentation_camera_description = camera_description.0.clone();
        segmentation_camera_description.name += "_segmentation";
        
        let (target, is_active) = match camera.target {
            RenderTarget::Image(_) => {
                (
                    image_table.create_render_target(
                        segmentation_camera_description.name.clone(),
                        segmentation_camera_description.width,
                        segmentation_camera_description.height,
                        &mut commands,
                        &mut images,
                        &render_device
                    ), true
                )
            }
            RenderTarget::Window(window) => (RenderTarget::Window(window), false),
            _ => unimplemented!(),
        };

        info!("Spawning Camera {}", segmentation_camera_description.name);
        
        commands.entity(entity).with_children(|parent| {

            parent.spawn((Camera3dBundle {
                camera: Camera {
                    order: 1,
                    target: target,
                    is_active: is_active,
                    clear_color: ClearColorConfig::Custom(Color::srgb_u8(0, 0, 0)),
                    ..default()
                },
                ..default()
            }, SegmentationCamera(segmentation_camera_description), RenderLayers::layer(1)));
        });

        // Force RGB cameras to render layer 0
        commands.entity(entity).insert(RenderLayers::layer(0));

    }
}

fn spawn_segmentation_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut object_table: ResMut<SegmentationDataTable>,
    query: Query<
        (Entity, &Handle<Mesh>, &mut SegmentationObject),
        (
            With<SegmentationObject>,
            Added<SegmentationObject>,
            Without<SegmentationObjectParent>,
        ),
    >,
) {
    for (entity, mesh_handle, segmentation_object) in query.iter() {
        commands.entity(entity).with_children(|parent| {
            parent.spawn((
                PbrBundle {
                    mesh: mesh_handle.clone(),
                    material: materials.add(StandardMaterial {
                        unlit: true,
                        base_color: object_table.color_of_object_assertive((*segmentation_object).clone()),
                        ..default()
                    }),
                    ..default()
                },
                RenderLayers::layer(1),
            ));
        });

        // Force user initialized entities to layer 0, and add marker indicating this object has a twin
        commands
            .entity(entity)
            .insert((RenderLayers::layer(0), SegmentationObjectParent));
    }
    info!("Spawned Segmentation Materials");
}

fn toggle_segmentation_view(
    keys: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Camera, With<SegmentationCamera>>,
) {
    for mut camera in camera_query.iter_mut() {
        match camera.target {
            RenderTarget::Window(_) => {
                if keys.just_pressed(KeyCode::Space) {
                    camera.is_active = true;
                }
            
                if keys.just_released(KeyCode::Space)  {
                    camera.is_active = false;
                }
            },
            _ => {},
        };
    }
}
