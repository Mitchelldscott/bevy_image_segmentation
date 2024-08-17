# bevy_image_segmentation

I like Bevy. I like Rust and I like Computer Vision, especially when I don't have to manually label the data in the training set.


I didn't see anyone else working on this so I'm trying to get an MVP going. 

Rendering the segmented view is pretty straight forward, just make an identical mesh with unlit texture for every spawned object with the segmentation marker (everything else is 'other'). However, there's no grayscale image texture so this is still RGBA and it is also on the GPU and needs to be exported.

[bevy_image_export](https://github.com/paulkre/bevy_image_export) is definitely the starting point, but will need to add some changes to support better file outputing. The changes are: 
1) Synchronize saving images from multiple cameras 
2) save files in sturcture that deep learning frameworks can [recognize](https://roboflow.com/formats).

## Goal interface

Modification of the [3D scene example](https://bevyengine.org/examples/3d-rendering/3d-scene/)

```
    //! A simple 3D scene with light shining over a cube sitting on a plane.

    use bevy::prelude::*;

    fn main() {
        App::new()
            .add_plugins(DefaultPlugins)
            .add_plugins(SegmentationAnnotationPlugin)
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
        commands.spawn(SegmentationObjectBundle { // Lightweight wrapper for PbrBundle
            mesh: meshes.add(Circle::new(4.0)),
            material: materials.add(Color::WHITE),
            transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            class_label: String::from("ground"), // Add a label to the object
            ..default()
        });
        // cube
        commands.spawn(SegmentationObjectBundle { // Lightweight wrapper for PbrBundle
            mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
            material: materials.add(Color::srgb_u8(124, 144, 255)),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            class_label: String::from("cuboid"), // Add a label to the object
            ..default()
        });
        // light
        commands.spawn(PointLightBundle {
            point_light: PointLight {
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..default()
        });
        // camera
        commands.spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        });
    }

```

## Resources
 - [Bevy Engine](https://bevyengine.org/)
 - [Bevy API](https://docs.rs/bevy/latest/bevy/index.html)
 - [Bevy Cheatbook](https://bevy-cheatbook.github.io/introduction.html)
 - [Bevy Render to Texture Example](https://bevyengine.org/examples/3d-rendering/render-to-texture/)
 - [Bevy Image Exporter](https://github.com/paulkre/bevy_image_export)
 - [BlenderProc](https://github.com/DLR-RM/BlenderProc)


> [GazeboSim SegmentationCamera](https://gazebosim.org/api/sensors/8/segmentationcamera_igngazebo.html) \
> [Git Discussion](https://github.com/gazebosim/gazebo-classic/issues/2933)

> [BlenderProc2 WhitePaper](https://doi.org/10.21105/joss.04901) \
> @article{Denninger2023, 
>   doi = {10.21105/joss.04901},
>   url = {https://doi.org/10.21105/joss.04901},
>   year = {2023},
>   publisher = {The Open Journal}, 
>   volume = {8},
>   number = {82},
>   pages = {4901}, 
>   author = {Maximilian Denninger and Dominik Winkelbauer and Martin Sundermeyer and Wout Boerdijk and Markus Knauer and Klaus H. Strobl and Matthias Humt and Rudolph Triebel},
>    title = {BlenderProc2: A Procedural Pipeline for Photorealistic Rendering}, 
>    journal = {Journal of Open Source Software}
> } 