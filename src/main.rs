use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(Msaa{ samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_scene(asset_server.load("models/eiffel_tower/scene.gltf#Scene0"));
    commands.spawn_scene(asset_server.load("models/the_eiffel/scene.gltf#Scene0"));
    commands.spawn_scene(asset_server.load("models/fontaine_wallace/scene.gltf#Scene0"));

    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });

    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-40.0, 0.0, -200.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}
