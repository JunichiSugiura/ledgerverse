use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .add_system(move_camera)
        .add_system(pan_camera)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let eiffel_tower_handle = asset_server.load("models/the_eiffel/scene.gltf#Scene0");
    let mercure_monte_sur_pegase_handler =
        asset_server.load("models/mercure_monte_sur_pegase/scene.gltf#Scene0");
    let spawn_plane_depth = 500.0f32;

    commands.spawn_scene(eiffel_tower_handle);
    commands.spawn_scene(mercure_monte_sur_pegase_handler);

    // light
    let theta = FRAC_PI_4;
    let light_transform = Mat4::from_euler(EulerRot::ZYX, 0.0, FRAC_PI_2, -theta);
    commands.spawn_bundle(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 100_000.0,
            shadows_enabled: true,
            ..Default::default()
        },
        transform: Transform::from_matrix(light_transform),
        ..Default::default()
    });

    // plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 2.0 * spawn_plane_depth,
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });

    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-40.0, 8.0, -200.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        })
        .insert(CameraController::default());
}

#[derive(Component)]
struct CameraController {
    pub enabled: bool,
    pub sensitivity: f32,
    pub key_forward: KeyCode,
    pub key_back: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
    pub key_run: KeyCode,
    pub walk_speed: f32,
    pub run_speed: f32,
    pub friction: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub velocity: Vec3,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: 0.5,
            key_forward: KeyCode::W,
            key_back: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::E,
            key_down: KeyCode::Q,
            key_run: KeyCode::LShift,
            walk_speed: 30.0,
            run_speed: 90.0,
            friction: 0.5,
            pitch: 0.0,
            yaw: 0.0,
            velocity: Vec3::ZERO,
        }
    }
}

#[derive(Component)]
struct Fontaine;

fn move_camera(
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
    key_input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    for (mut transform, mut options) in query.iter_mut() {
        if !options.enabled {
            continue;
        }

        let mut axis_input = Vec3::ZERO;
        if key_input.pressed(options.key_forward) {
            axis_input.z += 1.0
        }
        if key_input.pressed(options.key_back) {
            axis_input.z -= 1.0
        }
        if key_input.pressed(options.key_right) {
            axis_input.x += 1.0
        }
        if key_input.pressed(options.key_left) {
            axis_input.x -= 1.0
        }
        if key_input.pressed(options.key_up) {
            axis_input.y += 1.0
        }
        if key_input.pressed(options.key_down) {
            axis_input.y -= 1.0
        }

        if axis_input != Vec3::ZERO {
            let max_speed = if key_input.pressed(options.key_run) {
                options.run_speed
            } else {
                options.walk_speed
            };

            options.velocity = axis_input.normalize() * max_speed;
        } else {
            let friction = options.friction.clamp(0.0, 1.0);

            options.velocity *= 1.0 - friction;

            if options.velocity.length_squared() < 1e-6 {
                options.velocity = Vec3::ZERO;
            }
        }

        let forward = transform.forward();
        let right = transform.right();
        let dt = time.delta_seconds();

        transform.translation += options.velocity.x * dt * right
            + options.velocity.y * dt * Vec3::Y
            + options.velocity.z * dt * forward;
    }
}

fn pan_camera(
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
    mut mouse_events: EventReader<MouseMotion>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    let mut mouse_delta = Vec2::ZERO;
    for mouse_event in mouse_events.iter() {
        mouse_delta += mouse_event.delta;
    }

    for (mut transform, mut options) in query.iter_mut() {
        if mouse_delta != Vec2::ZERO {
            let (pitch, yaw) = (
                (options.pitch - mouse_delta.y * 0.5 * options.sensitivity * dt)
                    .clamp(-0.99 * FRAC_PI_2, 0.99 * FRAC_PI_2),
                options.yaw - mouse_delta.x * options.sensitivity * dt,
            );
            transform.rotation = Quat::from_euler(EulerRot::ZYX, 0.0, yaw, pitch);
            options.pitch = pitch;
            options.yaw = yaw;
        }
    }
}
