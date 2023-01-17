use bevy::{prelude::*, render::render_resource::Extent3d};

use crate::components::{VideoDecoder, VideoFrame};

pub fn apply_decode(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    decoders: Query<(Entity, &VideoDecoder)>,
) {
    for (entity, decoder) in decoders.iter() {
        let frame = decoder.take_frame_rgb8();
        if let Some(frame) = frame {
            let VideoFrame {
                buffer,
                width,
                height,
            } = frame;

            let image_handle = decoder.get_render_target();
            let image = match images.get_mut(&image_handle) {
                Some(image) => image,
                None => {
                    info!(
                        "Image gone. Removing video decoder from {:?} and stopping decode thread",
                        entity
                    );
                    commands.entity(entity).remove::<VideoDecoder>();
                    continue;
                }
            };

            if image.texture_descriptor.size.width != width as u32
                || image.texture_descriptor.size.height != height as u32
            {
                image.resize(Extent3d {
                    width: width as u32,
                    height: height as u32,
                    ..default()
                });
            }

            for (dest, src) in image.data.chunks_exact_mut(4).zip(buffer.chunks_exact(3)) {
                dest.copy_from_slice(&[src[2], src[1], src[0], 255]);
            }
        }
    }
}
