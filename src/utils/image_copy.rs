use bevy::{
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{self, NodeRunError, RenderGraph, RenderGraphContext, RenderLabel},
        render_resource::{
            Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor, Extent3d,
            ImageCopyBuffer, ImageDataLayout, Maintain, MapMode, 
        },
        renderer::{RenderContext, RenderDevice, RenderQueue},
        Extract, Render, RenderApp, RenderSet,
    },
};
use crossbeam_channel::Sender;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

// To communicate between the main world and the render world we need a channel.
// Since the main world and render world run in parallel, there will always be a frame of latency
// between the data sent from the render world and the data received in the main world
//
// frame n => render world sends data through the channel at the end of the frame
// frame n + 1 => main world receives the data
//
// Receiver and Sender are kept in resources because there is single camera and single target
// That's why there is single images role, if you want to differentiate images
// from different cameras, you should keep Receiver in ImageCopier and Sender in ImageToSave
// or send some id with data

/// Plugin for Render world part of work
pub struct ImageCopyPlugin;
impl Plugin for ImageCopyPlugin {
    fn build(&self, app: &mut App) {

        let render_app = app
            .sub_app_mut(RenderApp);

        let mut graph = render_app.world_mut().resource_mut::<RenderGraph>();
        graph.add_node(ImageCopy, ImageCopyDriver);
        graph.add_node_edge(bevy::render::graph::CameraDriverLabel, ImageCopy);

        render_app
            // Make ImageCopiers accessible in RenderWorld system and plugin
            .add_systems(ExtractSchedule, image_copy_extract)
            // Receives image data from buffer to channel
            // so we need to run it after the render graph is done
            .add_systems(Render, receive_image_from_buffer.after(RenderSet::Render));
    }
}

/// `ImageCopier` aggregator in `RenderWorld`
#[derive(Clone, Default, Resource, Deref, DerefMut)]
struct ImageCopiers(pub Vec<ImageCopier>);

/// Used by `ImageCopyDriver` for copying from render target to buffer
#[derive(Clone, Component)]
pub struct ImageCopier {
    buffer: Buffer,
    enabled: Arc<AtomicBool>,
    sender: Sender<Vec<u8>>,
    src_image: Handle<Image>,
}

impl ImageCopier {
    pub fn new(
        sender: Sender<Vec<u8>>,
        src_image: Handle<Image>,
        size: Extent3d,
        render_device: &RenderDevice,
    ) -> ImageCopier {
        let padded_bytes_per_row =
            RenderDevice::align_copy_bytes_per_row((size.width) as usize) * 4;

        let cpu_buffer = render_device.create_buffer(&BufferDescriptor {
            label: None,
            size: padded_bytes_per_row as u64 * size.height as u64,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        ImageCopier {
            buffer: cpu_buffer,
            sender: sender,
            src_image,
            enabled: Arc::new(AtomicBool::new(true)),
        }
    }

    pub fn enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }
}

/// Extracting `ImageCopier`s into render world, because `ImageCopyDriver` accesses them
fn image_copy_extract(mut commands: Commands, image_copiers: Extract<Query<&ImageCopier>>) {
    commands.insert_resource(ImageCopiers(
        image_copiers.iter().cloned().collect::<Vec<ImageCopier>>(),
    ));
}

/// `RenderGraph` label for `ImageCopyDriver`
#[derive(Debug, PartialEq, Eq, Clone, Hash, RenderLabel)]
struct ImageCopy;

/// `RenderGraph` node
#[derive(Default)]
struct ImageCopyDriver;

// Copies image content from render target to buffer
impl render_graph::Node for ImageCopyDriver {
    fn run(
        &self,
        _graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let image_copiers = world.get_resource::<ImageCopiers>().unwrap();
        let gpu_images = world
            .get_resource::<RenderAssets<bevy::render::texture::GpuImage>>()
            .unwrap();

        for image_copier in image_copiers.iter() {
            if !image_copier.enabled() {
                continue;
            }

            let src_image = gpu_images.get(&image_copier.src_image).unwrap();

            let mut encoder = render_context
                .render_device()
                .create_command_encoder(&CommandEncoderDescriptor::default());

            let block_dimensions = src_image.texture_format.block_dimensions();
            let block_size = src_image.texture_format.block_copy_size(None).unwrap();

            // Calculating correct size of image row because
            // copy_texture_to_buffer can copy image only by rows aligned wgpu::COPY_BYTES_PER_ROW_ALIGNMENT
            // That's why image in buffer can be little bit wider
            // This should be taken into account at copy from buffer stage
            let padded_bytes_per_row = RenderDevice::align_copy_bytes_per_row(
                (src_image.size.x as usize / block_dimensions.0 as usize) * block_size as usize,
            );

            let texture_extent = Extent3d {
                width: src_image.size.x,
                height: src_image.size.y,
                depth_or_array_layers: 1,
            };

            encoder.copy_texture_to_buffer(
                src_image.texture.as_image_copy(),
                ImageCopyBuffer {
                    buffer: &image_copier.buffer,
                    layout: ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(
                            std::num::NonZeroU32::new(padded_bytes_per_row as u32)
                                .unwrap()
                                .into(),
                        ),
                        rows_per_image: None,
                    },
                },
                texture_extent,
            );

            let render_queue = world.get_resource::<RenderQueue>().unwrap();
            render_queue.submit(std::iter::once(encoder.finish()));
        }

        Ok(())
    }
}

/// runs in render world after Render stage to send image from buffer via channel (receiver is in main world)
fn receive_image_from_buffer(
    image_copiers: Res<ImageCopiers>,
    render_device: Res<RenderDevice>,
) {
    for image_copier in image_copiers.0.iter() {
        if !image_copier.enabled() {
            continue;
        }

        // Finally time to get our data back from the gpu.
        // First we get a buffer slice which represents a chunk of the buffer (which we
        // can't access yet).
        // We want the whole thing so use unbounded range.
        let buffer_slice = image_copier.buffer.slice(..);

        // Now things get complicated. WebGPU, for safety reasons, only allows either the GPU
        // or CPU to access a buffer's contents at a time. We need to "map" the buffer which means
        // flipping ownership of the buffer over to the CPU and making access legal. We do this
        // with `BufferSlice::map_async`.
        //
        // The problem is that map_async is not an async function so we can't await it. What
        // we need to do instead is pass in a closure that will be executed when the slice is
        // either mapped or the mapping has failed.
        //
        // The problem with this is that we don't have a reliable way to wait in the main
        // code for the buffer to be mapped and even worse, calling get_mapped_range or
        // get_mapped_range_mut prematurely will cause a panic, not return an error.
        //
        // Using channels solves this as awaiting the receiving of a message from
        // the passed closure will force the outside code to wait. It also doesn't hurt
        // if the closure finishes before the outside code catches up as the message is
        // buffered and receiving will just pick that up.
        //
        // It may also be worth noting that although on native, the usage of asynchronous
        // channels is wholly unnecessary, for the sake of portability to Wasm
        // we'll use async channels that work on both native and Wasm.

        let (s, r) = crossbeam_channel::bounded(1);

        // Maps the buffer so it can be read on the cpu
        buffer_slice.map_async(MapMode::Read, move |r| match r {
            // This will execute once the gpu is ready, so after the call to poll()
            Ok(r) => s.send(r).expect("Failed to send map update"),
            Err(err) => panic!("Failed to map buffer {err}"),
        });

        // In order for the mapping to be completed, one of three things must happen.
        // One of those can be calling `Device::poll`. This isn't necessary on the web as devices
        // are polled automatically but natively, we need to make sure this happens manually.
        // `Maintain::Wait` will cause the thread to wait on native but not on WebGpu.

        // This blocks until the gpu is done executing everything
        render_device.poll(Maintain::wait()).panic_on_timeout();

        // This blocks until the buffer is mapped
        r.recv().expect("Failed to receive the map_async message");

        // This could fail on app exit, if Main world clears resources (including receiver) while Render world still renders
        let _ = image_copier.sender.send(buffer_slice.get_mapped_range().to_vec());

        // We need to make sure all `BufferView`'s are dropped before we do what we're about
        // to do.
        // Unmap so that we can copy to the staging buffer in the next iteration.
        image_copier.buffer.unmap();
    }
}