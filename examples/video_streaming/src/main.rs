use bevy::prelude::*;
use bevy_video::{nal_units, prelude::*};

#[derive(Resource)]
struct NalUnits(usize, Vec<Vec<u8>>);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(VideoPlugin)
        .insert_resource(NalUnits(
            0,
            nal_units(include_bytes!("./test.h264")) // https://software-download.name/sample-h264-video-file/download.html
                .map(|nal| nal.to_vec())
                .collect(),
        ))
        .add_startup_system(setup)
        .add_system(push_frame)
        .add_system(rotate_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (image_handle, video_decoder) = VideoDecoder::create(&mut images);

    // decoder
    commands.spawn(video_decoder);

    // plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
        material: materials.add(StandardMaterial {
            base_color_texture: Some(image_handle.clone()),
            ..default()
        }),
        ..default()
    });

    // // cube
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: materials.add(StandardMaterial {
    //         base_color_texture: Some(image_handle),
    //         ..default()
    //     }),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //     ..default()
    // });

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn push_frame(
    mut nal_units: ResMut<NalUnits>,
    decoders: Query<&VideoDecoder>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for _ in materials.iter_mut() {
        // otherwise the image wont update
    }

    if let Some(nal_unit) = nal_units.1.get(nal_units.0) {
        for decoder in decoders.iter() {
            decoder.add_video_packet(nal_unit.clone());
        }
    }
    nal_units.0 = (nal_units.0 + 1) % nal_units.1.len();
}

fn rotate_camera(time: Res<Time>, mut query: Query<&mut Transform, With<Camera>>) {
    let seconds = time.elapsed_seconds() * 0.2;
    for mut transform in query.iter_mut() {
        *transform = Transform::from_xyz(seconds.sin() * 5.0, 2.5, seconds.cos() * 5.0)
            .looking_at(Vec3::ZERO, Vec3::Y);
    }
}
