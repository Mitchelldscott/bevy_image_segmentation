
use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};

use bevy_image_export::{
    ImageExportBundle, ImageExportPlugin, ImageExportSettings, ImageExportSource,
};

#[derive(Component)]
struct MainCamera;

#[derive(Component)]
struct SegmentationCamera;

#[derive(Component)]
struct SegmentationObjectReflected;

#[derive(Component, Deref, PartialEq, Eq, Hash)]
pub struct SegmentationObject(pub String);

#[derive(Resource, Default)]
struct ImageExportHandle {
    handle: Handle<Image>,
}

fn random_color() -> Color {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let r: f32 = rng.gen(); // Generates a random float between 0.0 and 1.0
    let g: f32 = rng.gen();
    let b: f32 = rng.gen();

    Color::srgb(r, g, b)
}

// Define a resource to store all segmentation objects
#[derive(Resource)]
struct SegmentationDataTable {
    class_labels: Vec<String>,
    class_colors: Vec<Color>
}

impl SegmentationDataTable {
    fn index_of_label(&self, label: &String) -> Option<usize> {
        self.class_labels.iter().position(|n| n == label)
    }

    fn index_of_color(&self, color: &Color) -> Option<usize> {
        self.class_colors.iter().position(|c| c == color)
    }

    fn new_color(&self) -> Color {
        let mut new_color = random_color();
        while self.index_of_color(&new_color) != None {
            new_color = random_color();
        };
        new_color
    }

    fn label_id(&mut self, label: String) -> usize {
        match self.index_of_label(&label) {
            Some(id) => id,
            None => {
                self.class_labels.push(label);
                self.class_colors.push(self.new_color());
                self.class_labels.len() - 1
            }
        }
    }

    fn color_of_object_assertive(&mut self, label: String) -> Color {
        let index = self.label_id(label);
        self.class_colors[index].clone()
    }

    // fn color_id(&mut self, color: Color) -> Option<usize> {
    //     self.index_of_color(&color)
    // }

    // fn label_of(&self, index: usize) -> String {
    //     if index > self.class_labels.len() {return self.class_labels[0].clone()}
    //     self.class_labels[index].clone()
    // }

    // fn color_of(&self, index: usize) -> Color {
    //     if index > self.class_labels.len() {return self.class_colors[0].clone()}
    //     self.class_colors[index].clone()
    // }

    // fn color_of_object(&self, label: &String) -> Color {
    //     let index = match self.index_of_label(label) {
    //         Some(id) => id,
    //         None => 0,
    //     };
    //     self.class_colors[index].clone()
    // }

}

impl Default for SegmentationDataTable {
    fn default() -> Self {
        SegmentationDataTable {
            class_labels: vec![String::from("other")],
            class_colors: vec![Color::srgba(0.0, 0.0, 0.0, 0.0)]
        }
    }
}

// const WIDTH: u32 = 512;
// const HEIGHT: u32 = 512;

pub struct SegmentationPlugin;

impl Plugin for SegmentationPlugin {
    fn build(&self, app: &mut App) {
        let export_plugin = ImageExportPlugin::default();
        let export_threads = export_plugin.threads.clone();

        app
            .add_plugins(export_plugin)
            .add_systems(Startup, (setup, spawn_segmentation_children).chain())
            .add_systems(
                Update,
                toggle_segmentation_display.run_if(resource_changed::<ButtonInput<KeyCode>>),
            )
            .add_systems(Update, save_image);

        export_threads.finish();
    }
}

fn setup(
    mut commands: Commands,
    windows: Query<&mut Window>,
    mut images: ResMut<Assets<Image>>,
) {

    let window = windows.single();
    // window.resolution.set(WIDTH as f32, HEIGHT as f32);

    let output_image_handle = {
        let size = Extent3d {
            width: window.width() as u32, // WIDTH
            height: window.height() as u32, // WIDTH
            depth_or_array_layers: 1,
        };
        let mut export_image = Image {
            texture_descriptor: TextureDescriptor {
                label: "SegmentationTexture".into(),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::COPY_SRC
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..default()
        };

        export_image.resize(size);

        images.add(export_image)
    };

    // Cameras
    commands
        .spawn((
            Camera3dBundle {
                transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
            MainCamera,
            RenderLayers::layer(0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3dBundle {
                    camera: Camera {
                        order: 1,
                        is_active: false,
                        clear_color: ClearColorConfig::None,
                        ..default()
                    },
                    ..default()
                },
                SegmentationCamera,
                RenderLayers::layer(1),
            ));
            parent.spawn((
                Camera3dBundle {
                    camera: Camera {
                        target: RenderTarget::Image(output_image_handle.clone()),
                        ..default()
                    },
                    ..default()
                },
                RenderLayers::layer(1),
            ));
        });

    commands.init_resource::<SegmentationDataTable>();
    commands.insert_resource(ImageExportHandle {
        handle: output_image_handle,
    });

}

fn spawn_segmentation_children(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut object_table: ResMut<SegmentationDataTable>,
    query: Query<
        (Entity, &Handle<Mesh>, &mut SegmentationObject),
        (
            With<SegmentationObject>,
            Added<SegmentationObject>,
            Without<SegmentationObjectReflected>,
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

        commands
            .entity(entity)
            .insert((RenderLayers::layer(0), SegmentationObjectReflected));
    }

    println!("Done Spawning Segmentation Children");
}

#[derive(Component)]
pub struct ExportBundleMarker;

fn save_image(
    mut commands: Commands,
    mut export_sources: ResMut<Assets<ImageExportSource>>,
    input: Res<ButtonInput<KeyCode>>,
    output_texture: ResMut<ImageExportHandle>,
    export_bundles: Query<Entity, With<ExportBundleMarker>>,
) {
    if input.just_pressed(KeyCode::KeyS) {
        commands.spawn((
            ImageExportBundle {
                source: export_sources.add(output_texture.handle.clone()),
                settings: ImageExportSettings {
                    output_dir: format!("segmentation"),
                    extension: "png".into(),
                },
            },
            ExportBundleMarker,
        ));
    }

    for exporter in &export_bundles {
        commands.entity(exporter).despawn();
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
