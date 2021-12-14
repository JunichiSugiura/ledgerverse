use bevy::prelude::*;
use bevy::{input::mouse::MouseMotion};

fn main() {
    App::new()
        .insert_resource(Msaa{ samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(walk)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let eiffel_tower_handle = asset_server.load("models/the_eiffel/scene.gltf#Scene0");
    let fontaine_wallace_handle = asset_server.load("models/fontaine_wallace/scene.gltf#Scene0");

    commands.spawn_bundle((Transform::from_xyz(0.0, -0.0, 0.0), GlobalTransform::identity())).with_children(|parent| {
        parent.spawn_scene(eiffel_tower_handle);

        parent.spawn_scene(fontaine_wallace_handle);

        // try to reposition scene;
        // parent.spawn_bundle(SceneBundle{
        //     scene: fontaine_wallace_handle,
        //     transform: Transform::from_xyz(0.0, -2.0, 0.0),
        //     ..Default::default()
        // });

        // light
        parent.spawn_bundle(PointLightBundle {
            transform: Transform::from_xyz(4.0, 8.0, 4.0),
            ..Default::default()
            }
        );

        // camera
        parent.spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-40.0, 8.0, -200.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        }).insert(MainCamera);
    });
}

#[derive(Component)]
struct MainCamera;

fn walk(
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    keys: Res<Input<KeyCode>>,
    // mut mouse_events: EventReader<MouseMotion>,
) {
    let mut camera_transform = camera_query.single_mut();
    let step = 1.0;

    if keys.pressed(KeyCode::A) {
        camera_transform.translation.x = camera_transform.translation.x + step;
    };

    if keys.pressed(KeyCode::D) {
        camera_transform.translation.x = camera_transform.translation.x - step;
    }

    if keys.pressed(KeyCode::W) {
        camera_transform.translation.z = camera_transform.translation.z + step;
    }

    if keys.pressed(KeyCode::S) {
        camera_transform.translation.z = camera_transform.translation.z - step;
    }

    // for ev in mouse_events.iter() {
    //     println!("Mouse moved: X: {} px, Y: {} px", ev.delta.x, ev.delta.y);
    // }

    // camera_transform.translation.y = camera_transform.translation.y + 0.1;
}
