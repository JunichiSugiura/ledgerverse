use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use ledger::TransportNativeHID;
use ledger_transport::APDUTransport;
use ledger_zondax_generic::*;
use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .insert_resource(LedgerConnect::new())
        .add_startup_system(setup)
        .add_system(move_camera)
        .add_system(pan_camera)
        .add_system(update_text_position)
        .add_system(connect_device)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn_scene(asset_server.load("models/the_eiffel/scene.gltf#Scene0"));
    commands.spawn_scene(asset_server.load("models/mercure_monte_sur_pegase/scene.gltf#Scene0"));

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
    let spawn_plane_depth = 500.0f32;
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 2.0 * spawn_plane_depth,
        })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..Default::default()
    });

    // ui camera
    commands.spawn_bundle(UiCameraBundle::default());

    // text
    let font = asset_server.load("fonts/Roboto_Slab/RobotoSlab-VariableFont_wght.ttf");
    commands
        .spawn_bundle(TextBundle {
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    bottom: Val::Px(100.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                size: Size {
                    width: Val::Px(200.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text::with_section(
                "Press \"I\" for device info".to_string(),
                TextStyle {
                    font: font.clone(),
                    font_size: 14.0,
                    color: Color::WHITE,
                },
                TextAlignment {
                    ..Default::default()
                },
            ),
            ..Default::default()
        })
        .insert(ConnectText);

    // camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(200.0, 8.0, 40.0).looking_at(Vec3::ZERO, Vec3::Y),
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

#[derive(Component)]
struct ConnectText;

#[derive(Component)]
struct DeviceInfo;

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

        if transform.translation.y < 8.0 {
            transform.translation.y = 8.0;
        }
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

fn update_text_position(
    windows: Res<Windows>,
    mut text_query: Query<(&mut Style, &CalculatedSize), With<ConnectText>>,
    mesh_query: Query<&Transform, With<Handle<Mesh>>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<CameraController>>,
) {
    for (camera, camera_transform) in camera_query.iter() {
        for mesh_position in mesh_query.iter() {
            for (mut style, calculated) in text_query.iter_mut() {
                match camera.world_to_screen(&windows, camera_transform, mesh_position.translation)
                {
                    Some(coords) => {
                        style.position.left = Val::Px(coords.x - calculated.size.width / 2.0);
                        style.position.bottom = Val::Px(coords.y - calculated.size.height / 2.0);
                        // let c = ((coords.x.powf(2.0) + coords.y.powf(2.0)) * -1.0).sqrt();
                        // println!("x: {}, y: {}, c: {}", coords.x, coords.y, c);
                        // text_style.font_size = coords.y
                    }
                    None => {
                        // A hack to hide the text when the cube is behind the camera
                        style.position.bottom = Val::Px(-1000.0);
                    }
                }
            }
        }
    }
}

fn connect_device(key_input: Res<Input<KeyCode>>, mut connect: ResMut<LedgerConnect>) {
    if key_input.pressed(KeyCode::I) {
        connect.run();
    }
}

struct LedgerConnect {
    runtime: tokio::runtime::Runtime,
}

impl LedgerConnect {
    pub fn new() -> Self {
        Self {
            runtime: tokio::runtime::Runtime::new().expect("init async runtime"),
        }
    }

    pub fn run(&mut self) {
        self.runtime.spawn(async move {
            let hid_wrapper = TransportNativeHID::new();

            tokio::spawn(async move {
                match hid_wrapper {
                    Ok(wrapper) => {
                        let transport = APDUTransport::new(wrapper);
                        let device_info: Option<ledger_zondax_generic::DeviceInfo>;
                        let app_info: Option<ledger_zondax_generic::AppInfo>;

                        match get_device_info(&transport).await {
                            Ok(device) => {
                                // println!("{:#?}", device);
                                device_info = Some(device);
                            }
                            Err(err) => {
                                eprintln!("{}", err);
                                device_info = None;
                            }
                        }

                        match get_app_info(&transport).await {
                            Ok(app) => {
                                // println!("{:#?}", app);
                                app_info = Some(app);
                            }
                            Err(err) => {
                                eprintln!("{}", err);
                                app_info = None;
                            }
                        }

                        if let Some(info) = device_info {
                            println!("");
                            println!("///////////////// Device Info /////////////////////////");
                            println!("se_version: {}", info.clone().se_version);
                            println!("mcu_version: {}", info.mcu_version);
                        }
                        if let Some(info) = app_info {
                            println!("");
                            println!("///////////////// App Info /////////////////////////");
                            println!("app_name: {}", info.clone().app_name);
                            println!("app_version: {}", info.app_version);
                        }
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        // TODO Quit app
                    }
                }
            });
        });
    }
}
