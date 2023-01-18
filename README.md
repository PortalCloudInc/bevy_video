# Bevy AVC

Stream video to your Bevy app!

```rust
use bevy::prelude::*;
use bevy_video::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(VideoPlugin)
        .add_startup_system(setup)
        .add_system(push_frame)
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    let (image_handle, video_decoder) = VideoDecoder::create(&mut images);

    // decoder
    commands.spawn(video_decoder);

    // ...
}

fn push_frame(
    decoders: Query<&VideoDecoder>,
    mut materials: ResMut<Assets<MaterialThatUsesTheImage>>,
) {
    for _ in materials.iter_mut() {
        // otherwise the image on screen wont update
    }
    for decoder in decoders.iter() {
        decoder.add_video_packet(/* Vec<u8> representing an H.264 packet */);
        // Note: packets are decoded asynchronously in another thread
        // The `Image` will update automatically
    }
}
```
