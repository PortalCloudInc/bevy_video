use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};

use bevy::{
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    },
};
use openh264::decoder::{Decoder, DecoderConfig};

enum DecoderMessage {
    Frame(Vec<u8>),
    Stop,
}

pub struct VideoFrame {
    pub buffer: Vec<u8>,
    pub width: usize,
    pub height: usize,
}

#[derive(Component)]
pub struct VideoDecoder {
    sender: Mutex<Sender<DecoderMessage>>,
    next_frame_rgb8: Arc<Mutex<Option<VideoFrame>>>,
    render_target: Handle<Image>,
}

impl VideoDecoder {
    pub fn create(images: &mut ResMut<Assets<Image>>) -> (Handle<Image>, VideoDecoder) {
        let render_target = images.add(Self::create_image(12, 12));
        let (sender, receiver) = channel::<DecoderMessage>();
        let next_frame_rgb8 = Arc::new(Mutex::new(None));

        std::thread::spawn({
            let next_frame_rgb8 = next_frame_rgb8.clone();
            move || {
                let cfg = DecoderConfig::new();
                let mut decoder = Decoder::with_config(cfg).expect("Failed to create AVC decoder");
                for video_packet in receiver {
                    let video_packet = match video_packet {
                        DecoderMessage::Frame(video_packet) => video_packet,
                        DecoderMessage::Stop => return,
                    };
                    let decoded_yuv = decoder.decode(video_packet.as_slice());
                    let decoded_yuv = match decoded_yuv {
                        Ok(decoded_yuv) => decoded_yuv,
                        Err(e) => {
                            error!("Failed to decode frame: {}", e);
                            continue;
                        }
                    };
                    let Some(decoded_yuv) = decoded_yuv else { continue };
                    let (width, height) = decoded_yuv.dimension_rgb();
                    let mut buffer = vec![0; width * height * 3];

                    // TODO: Don't convert YUV -> RGB -> BGRA, just make something for YUV -> BGRA
                    decoded_yuv.write_rgb8(buffer.as_mut_slice());

                    let frame = VideoFrame {
                        buffer,
                        width,
                        height,
                    };

                    next_frame_rgb8.lock().unwrap().replace(frame);
                }
            }
        });

        let video_decoder = Self {
            sender: Mutex::new(sender),
            next_frame_rgb8,
            render_target: render_target.clone_weak(),
        };

        (render_target, video_decoder)
    }

    fn create_image(width: u32, height: u32) -> Image {
        let size = Extent3d {
            width,
            height,
            ..default()
        };

        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: Some("Video stream render target"),
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::COPY_DST
                    | TextureUsages::TEXTURE_BINDING
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[TextureFormat::Bgra8UnormSrgb],
            },
            ..default()
        };
        image.resize(size);
        image
    }

    pub fn add_video_packet(&self, video_packet: Vec<u8>) {
        self.sender
            .lock()
            .expect("Could not get lock on sender")
            .send(DecoderMessage::Frame(video_packet))
            .expect("Could not send packet to decoder thread");
    }

    pub(crate) fn take_frame_rgb8(&self) -> Option<VideoFrame> {
        self.next_frame_rgb8.lock().unwrap().take()
    }

    pub fn get_render_target(&self) -> Handle<Image> {
        self.render_target.clone_weak()
    }
}

impl Drop for VideoDecoder {
    fn drop(&mut self) {
        self.sender
            .lock()
            .expect("Could not get lock on sender")
            .send(DecoderMessage::Stop)
            .expect("Could not send stop message to decoder thread");
    }
}
