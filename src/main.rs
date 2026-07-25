
use bevy::audio::Volume;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::{FRAC_PI_2};
use std::num::NonZero;
use bevy::input::mouse::MouseMotion;
use bevy::light::NotShadowCaster;
use bevy_rapier3d::rapier::prelude::CollisionEventFlags;
use rand::RngExt;
use bevy_rich_text3d::{Text3d, Text3dPlugin, Text3dStyling, TextAlign, TextAtlas};
const BUMP: f32 = 2.5;

const BACKGROUND_COLOR: Color = Color::srgb(0.7, 0.8, 0.7);
const DEAD_BALL: Color = Color::srgb(0.9, 0.0, 0.9);
const LIVE_BALL: Color = Color::srgb(1.0, 0.0, 0.0);
const CONE_COLOR: Color = Color::srgb(1.0, 0.0, 1.0);
const DISK_COLOR: Color = Color::srgb(0.0, 0.9, 0.5);
const BOX_COLOR: Color = Color::srgb(0.0, 0.0, 1.0);
const BOX_COLOR_TRANSPARENT: Color = Color::srgba(0.0, 0.0, 1.0, 0.2);
const LIGHT_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const BARRIER_COLOR: Color = Color::srgb(1.0, 0.64, 0.0);
const TARGET_COLOR: Color = Color::srgb(255.0/255.0, 105.0/255.0, 180.0/255.0);
const FLOOR_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);
const SCOREBOARD_COLOR: Color = Color::srgb(0.5, 0.5, 0.0);
const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
const CYLINDER_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
const _CYLINDER_HALF_HEIGHT: f32 = 2.0;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Initialize the Rapier physics engine and the debug renderer
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
//        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(Text3dPlugin{
            default_atlas_dimension: (1024, 1024),
            load_system_fonts: true,
            ..Default::default()
        })
        .add_systems(Startup, setup_physics)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(ScoreBoard::new())
        .insert_resource(GlobalVolume::new(Volume::Linear(0.25)))
        .add_systems(Update, (
            toggle_overhead_camera.run_if(input_just_pressed(KeyCode::KeyO)),
            toggle_wind.run_if(input_just_pressed(KeyCode::KeyW)),
            drop_a_ball.run_if(input_just_pressed(KeyCode::Enter)),
            drop_a_ball.run_if(input_just_pressed(KeyCode::NumpadEnter)),
            impulse.run_if(input_just_pressed(KeyCode::Space)),
            impulse.run_if(input_just_pressed(KeyCode::Numpad5)),
            impulse.run_if(input_just_pressed(KeyCode::ArrowLeft)),
            impulse.run_if(input_just_pressed(KeyCode::Numpad4)),
            impulse.run_if(input_just_pressed(KeyCode::ArrowRight)),
            impulse.run_if(input_just_pressed(KeyCode::Numpad6)),
            impulse.run_if(input_just_pressed(KeyCode::ArrowDown)),
            impulse.run_if(input_just_pressed(KeyCode::Numpad2)),
            impulse.run_if(input_just_pressed(KeyCode::ArrowUp)),
            impulse.run_if(input_just_pressed(KeyCode::Numpad8)),
            new_game.run_if(input_just_pressed(KeyCode::KeyG)),
            handle_sensor_events,
            handle_drop_toys,
            handle_sound,
            cleanup_fallen_entities,
            update_scoreboard.run_if(resource_changed::<ScoreBoard>),
            // mouse_look_system.run_if(|mouse: Res<ButtonInput<MouseButton>>| mouse.pressed(MouseButton::Left)),
        )
    )
    .add_message::<DropToys>()
    .add_message::<Sound>()
    .run();
}
enum SoundType {
    Win,
    Lose,
}
#[derive(Message)]
struct Sound {
    sound_type: SoundType,
}
#[derive(Message)]
struct DropToys {
}

#[derive(Component)]
struct Score {
}

#[derive(Component)]
struct Post {
}
#[derive(Clone, Copy, Debug)]
enum BouncyBallStatus {
    Live,
    Dead,
}
#[derive(Component, Clone, Copy, Debug)]
struct BouncyBall {
    status: BouncyBallStatus,
}

#[derive(Component)]
struct SensorChild {
}

#[derive(Component)]
struct PointValue {
    value: i32,
}

#[derive(Component)]
struct Toy {
    dynamic: bool,
}

#[derive(Resource)]
struct ScoreBoard {
    score: i32,
    round: i32,
    total: i32,
}

impl ScoreBoard {
    fn new() -> Self {
        Self{score: 0, round: 0, total: 0}
    }
    fn hit(&mut self, incr: i32) {
        self.score += incr;
        self.total += incr;
    }
    fn new_round(&mut self) {
        self.score = 0;
        self.round += 1;
    }
    fn reset(&mut self) {
        self.score = 0;
        self.round = 0;
        self.total = 0;
    }
}
#[derive(Component)]
struct CameraController {
    sensitivity: f32,
    pitch: f32,
    yaw: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            sensitivity: 0.002,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}
#[derive(Event)]
struct Unselect {
}

fn toggle_wind(
    mut force_query: Query<&mut ExternalForce>,
) {
    for mut force in force_query.iter_mut() {
        if force.force.x > 0.0 {
            force.force.x = 0.0;
        } else {
            force.force.x = 2.0;
        }
    }
}

fn toggle_overhead_camera(
    mut q_camera: Query<&mut Transform, With<CameraController>>,
    mut q_light: Query<&mut Transform, (With<PointLight>, Without<CameraController>)>,
) {
    for mut transform in q_camera.iter_mut() {
        if transform.translation.z < 25.0 {
            *transform = Transform::from_xyz(0.0, 10.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y);
        } else {
            *transform = Transform::from_xyz(0.0, 30.0, 0.2).looking_at(Vec3::ZERO, Vec3::Y);
        }
    }
    // Move the light, too
    for mut transform in q_light.iter_mut() {
        if transform.translation.z < 25.0 {
            transform.translation.y = 20.0;
            transform.translation.z = 10.0;
        } else {
            transform.translation.y = 20.0;
            transform.translation.z = 1.0;
        }
    }
}
fn mouse_look_system(
    mut mouse_motion_events: MessageReader<MouseMotion>,
) {
    let mut delta_x = 0.0;
    let mut delta_y = 0.0;
    let scale = 0.05;
    for event in mouse_motion_events.read() {
        delta_x += event.delta.x*scale;
        delta_y += event.delta.y*scale;
    }

    if delta_x == 0.0 && delta_y == 0.0 {
        return;
    }
}
fn handle_sensor_events(
    mut messages: ResMut<Messages<CollisionEvent>>,
    // mut commands: Commands,
    ball_query: Query<(Entity, &BouncyBall), With<BouncyBall>>,
    mut toy_query: Query<(Entity, &mut Toy, &mut RigidBody, &mut PointValue), (With<Toy>, Without<SensorChild>)>,
    mut sensor_query: Query<(Entity, &ChildOf, &mut PointValue), (With<SensorChild>, Without<Toy>)>,
) {
    for event in messages.drain() {
        match event {
            CollisionEvent::Started(entity1, entity2, flags) => {
                for (ball_entity, bouncy_ball) in ball_query.iter() {
//                    println!("Collision {:?} {:?}. Ball is: {:?} and it is {:?}", entity1, entity2, ball_entity, bouncy_ball);
                    let toy = if ball_entity == entity2 {
                        entity1
                    } else {
                        entity2
                    };
                    if flags.contains(CollisionEventFlags::SENSOR) {
                        // No more points after a sensor is touched
                        let (child_entity, parent_entity, mut child_point_value) = sensor_query.get_mut(toy).unwrap();
                        let (toy_entity, mut toy_component, mut rigid_body, mut parent_point_value) = toy_query.get_mut(parent_entity.0).unwrap();
                        println!("Parent entity: {:?} (pointvalue: {:?}), child: {:?} (pointvalue: {:?})",
                                 parent_entity.0, parent_point_value.value, child_entity.entity(), child_point_value.value);
                        parent_point_value.value += child_point_value.value;
                        child_point_value.value = 0;
                        // But add the points to the parent of the toy
                        println!("Sensor event");
                        return;
                    }
                    match bouncy_ball.status {
                        BouncyBallStatus::Live => {
                            for (toy_entity, mut toy_component, mut rigid_body, point_value) in toy_query.iter_mut() {
                                if toy_entity == toy && !toy_component.dynamic {
 //                                   println!("Make toy dynamic");
                                    *rigid_body = RigidBody::Dynamic;
                                    toy_component.dynamic = true;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            CollisionEvent::Stopped(_entity1, _entity2, _flags) => {
//                println!("Something left the sensor: {:?} and {:?}", entity1, entity2);
            }
        }
    }
}

fn cleanup_fallen_entities(
    mut commands: Commands,
    ball_query: Query<&BouncyBall>,
    shapes: Query<(Entity, &mut Transform, &PointValue)>,
    mut scoreboard: ResMut<ScoreBoard>,
) {
    // score and cleanup old toys and balls that are out of range
    for (entity, shape, point_value) in shapes.iter() {
        if shape.translation.y < -15.0 {
            // Don't make a sound for zero point value
            for ball in ball_query.iter() {
                match ball.status {
                    BouncyBallStatus::Live => {
                        if point_value.value != 0 {
                            scoreboard.hit(point_value.value);
                            commands.write_message(
                                Sound {
                                    sound_type: if point_value.value < 0 { SoundType::Lose }
                                    else { SoundType::Win }
                                });
                        }
                    }
                    _ => {}
                }
            }
            commands.entity(entity).despawn();
            println!("Entity despawned {} points", point_value.value);
        }
    }
}

// Drop a Mouse which has a lot of facets
fn drop_a_mouse(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        RigidBody::Dynamic,
        Friction::new(0.0),
        Restitution::new(0.1),
        Toy {dynamic: false},
        PointValue { value: 10 },
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/Mouse.glb#Collection"))),
        AsyncSceneCollider::default(),
        Transform::from_xyz(-4.0, 5.0, 5.5).with_scale(Vec3::splat(2.0)),
    ));
}
fn handle_sound(
    mut messages: MessageReader<Sound>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for event in messages.read() {
        match event.sound_type {
            SoundType::Win => {
                commands.spawn((
                    AudioPlayer::new(asset_server.load("audio/beep.ogg")),
                    PlaybackSettings::ONCE,
                ));
            }
            SoundType::Lose => {
                commands.spawn((
                    AudioPlayer::new(asset_server.load("audio/buzzer.ogg")),
                    PlaybackSettings::ONCE,
                ));
            }
        }
    }
}
fn handle_drop_toys(
    mut messages: MessageReader<DropToys>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for _event in messages.read() {
        let mut rng = rand::rng();
        // A Target
        commands.spawn((
            Toy { dynamic: false },
            RigidBody::Fixed,
            PointValue { value: 35 },
            ExternalImpulse::default(), // For when this becomes dynamic
            Friction::new(0.1),
            Restitution::new(0.1),
            Collider::cylinder(0.1, 0.5),
            Mesh3d(meshes.add(Mesh::from(Cylinder::new(0.50, 0.2)))),
            MeshMaterial3d(materials.add(TARGET_COLOR)),
            Transform::from_xyz(-14.0, 3.0, 4.0).with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
        ));
        commands.spawn((
            Toy { dynamic: false },
            RigidBody::Fixed,
            PointValue { value: 45 },
            ExternalImpulse::default(), // For when this becomes dynamic
            Friction::new(0.1),
            Restitution::new(0.1),
            Collider::cuboid(0.25, 1.0, 1.0),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.50, 1.5, 1.5)))),
            MeshMaterial3d(materials.add(TARGET_COLOR)),
            Transform::from_xyz(-13.9, 7.0, -4.0),
        ));
        // Local cones
        for n in 0..2 {
            commands.spawn((
                Toy { dynamic: true },
                RigidBody::Dynamic,
                Friction::new(0.9),
                Restitution::new(0.1),
                Mesh3d(meshes.add(Mesh::from(Cone::new(0.75, 2.0)))),
                MeshMaterial3d(materials.add(CONE_COLOR)),
                ExternalImpulse::default(),
                PointValue { value: 15 },
                // Lower the Damping for a more advanced game
                // Damping {
                //     linear_damping: 0.2,
                //     angular_damping: 0.2,
                // },
                Collider::cone(1.0, 0.75),
                Transform::from_xyz(rng.random_range(-12.0..12.0),
                                    10.0 + rng.random_range(0.0..10.0),
                                    rng.random_range(-9.0..9.0)),
                ExternalForce::default(),
            ));
        }
        // Local disks
        for n in 0..4 {
            commands.spawn((
                RigidBody::Dynamic,
                Toy { dynamic: true },
                Friction::new(0.2),
                Restitution::new(0.1),
                Mesh3d(meshes.add(Mesh::from(Cylinder::new(0.75, 0.6)))),
                MeshMaterial3d(materials.add(DISK_COLOR)),
                ExternalImpulse::default(),
                PointValue { value: 15 },
                // Lower the Damping for a more advanced game
                // Damping {
                //     linear_damping: 0.2,
                //     angular_damping: 0.2,
                // },
                Collider::cylinder(0.3, 0.75),
                Transform::from_xyz(rng.random_range(-12.0..12.0),
                                    10.0 + rng.random_range(0.0..10.0),
                                    rng.random_range(-9.0..9.0)),
                ExternalForce::default(),
            ));
        }
        // Dip
        commands.spawn((
            RigidBody::Dynamic,
            Toy { dynamic: true },
            ExternalImpulse::default(),
            Friction::new(0.0),
            Restitution::new(0.1),
            PointValue { value: 10 },
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/dip.glb#collection"))),
            AsyncSceneCollider::default(),
            Transform::from_xyz(-9.0, 8.0, 3.0).with_scale(Vec3::splat(0.5)),
        )).with_children(|parent| {
            parent.spawn((
                SensorChild {},
                Collider::ball(0.1),
                Sensor,
                PointValue { value: 25 },
                ActiveEvents::COLLISION_EVENTS,
                Transform::from_xyz(0.0, 0.2, 0.0),
            ));
        });

        // Boxes
        for n in 0..6 {
            commands.spawn((
                RigidBody::Dynamic,
                Toy { dynamic: true },
                Friction::new(0.2),
                Restitution::new(0.1),
                Mesh3d(meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: if rng.random_bool(0.30) { BOX_COLOR_TRANSPARENT } else { BOX_COLOR },
                    alpha_mode: AlphaMode::Blend,
                    ..default()
                })),
                NotShadowCaster,
                ExternalImpulse::default(),
                PointValue { value: 15 },
                // Lower the Damping for a more advanced game
                // Damping {
                //     linear_damping: 0.2,
                //     angular_damping: 0.2,
                // },
                Collider::cuboid(0.5, 0.5, 0.5),
                Transform::from_xyz(rng.random_range(-12.0..12.0),
                                    10.0 + rng.random_range(0.0..10.0),
                                    rng.random_range(-9.0..9.0)),
                ExternalForce::default(),
            ));
        };
        // Doughnut (torus)
        commands.spawn((
            RigidBody::Dynamic,
            Toy { dynamic: true },
            Friction::new(0.0),
            Restitution::new(0.1),
            ExternalImpulse::default(),
            PointValue { value: 10 },
            ColliderMassProperties::Density(0.25),
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/doughnut.glb#collection"))),
            AsyncSceneCollider::default(),
            Transform::from_xyz(10.0, 12.0, 7.0).with_scale(Vec3::splat(1.0)),
        )).with_children(|parent| {
            parent.spawn((
                SensorChild {},
                Collider::ball(0.1),
                PointValue { value: 20 },
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                Transform::from_xyz(0.0, 0.2, 0.0),
            ));
        });

        // Cylinder
        for n in 0..6 {
            commands.spawn((
                RigidBody::Dynamic,
                Toy { dynamic: true },
                Friction::new(0.8),
                Restitution::new(0.1),
                ExternalImpulse::default(),
                PointValue { value: 5 },
                WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cylinder.glb#collection"))),
                AsyncSceneCollider::default(),
                Transform::from_xyz(rng.random_range(-12.0..12.0), 4.0, rng.random_range(-9.0..9.0)).with_scale(Vec3::splat(0.75)),
            ));
        }
        // Bumpy sphere
        commands.spawn((
            RigidBody::Dynamic,
            Toy { dynamic: true },
            Friction::new(0.0),
            Restitution::new(0.1),
            ExternalImpulse::default(),
            PointValue { value: 5 },
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/bumpy.glb#collection"))),
            AsyncSceneCollider::default(),
            Transform::from_xyz(10.0, 12.0, -7.0),
        ));
    }
}
fn new_game(
    mut old_balls: Query<Entity, With<BouncyBall>>,
    mut old_toys: Query<Entity, With<Toy>>,
    mut scoreboard: ResMut<ScoreBoard>,
    mut commands: Commands,
) {
    scoreboard.reset();
    for entity in old_balls.iter_mut() {
        commands.entity(entity).despawn();
    }
    for entity in old_toys.iter_mut() {
        commands.entity(entity).despawn();
    }
    commands.write_message(DropToys{});
}
fn drop_a_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scoreboard: ResMut<ScoreBoard>,
    mut query: Query<(&mut BouncyBall, &mut PointValue, &MeshMaterial3d<StandardMaterial>), With<BouncyBall>>,
) {
    // Make any live balls dead, usually only one
    for (mut bouncyball, mut point_value, mut material_handle) in query.iter_mut() {
        match bouncyball.status {
            BouncyBallStatus::Live => {
                bouncyball.status = BouncyBallStatus::Dead;
                point_value.value = 2;
                if let Some(mut material) = materials.get_mut(material_handle) {
                    material.base_color = DEAD_BALL;
                }
//                println!("Live - > Dead");

            }
            _ => {}
        }
    }
    let mut rng = rand::rng();
    let x_pos: f32 = rng.random_range(-12.0..0.);
    let y_pos: f32 = rng.random_range(15.0..25.);
    let z_pos: f32 = rng.random_range(-9.0..9.0);
    // Spawn a Dynamic Bouncing Ball
    scoreboard.new_round();
    let entity = commands.spawn((
        BouncyBall{status: BouncyBallStatus::Live},
        RigidBody::Dynamic,
        PointValue{value: -10},
        ActiveEvents::COLLISION_EVENTS,
        // Lower the Damping for a more advanced game
        Damping {
            linear_damping: 0.2,
            angular_damping: 0.2,
        },
        ColliderMassProperties::Density(2.0),
        //        Ccd::enabled(), // doesn't help sticky balls
        Collider::ball(0.5),
        // Adding restitution makes the ball bounce
        Restitution::new(1.0),
        //        GravityScale(2.0),
        ExternalImpulse::default(),
        Transform::from_xyz(x_pos, y_pos, z_pos),
        Velocity::linear(Vec3::new(2.0, 0.0, 0.0)),
        Visibility::default(),
        ExternalForce::default(),
        Mesh3d(meshes.add(Mesh::from(Sphere::new(0.5)))),
        MeshMaterial3d(materials.add(LIVE_BALL)),
    )).id();
    commands.trigger(Unselect{});
}

fn impulse(
    mut balls: Query<(&mut ExternalImpulse, &BouncyBall), (With<BouncyBall>)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Just interested in the live ball
    for (mut impulse, ball) in balls.iter_mut() {
        match ball.status  {
            BouncyBallStatus::Live => {
                // See which key was pressed
                for key in keyboard_input.get_just_pressed() {
                    match key {
                        KeyCode::Space => {
                            impulse.impulse = Vec3::new(0.0, BUMP*2.0, 0.0);
                        }
                        KeyCode::Numpad5 => {
                            impulse.impulse = Vec3::new(0.0, BUMP*2.0, 0.0);
                        }
                        KeyCode::ArrowLeft => {
                            impulse.impulse = Vec3::new(-BUMP, 0.0, 0.0);
                        }
                        KeyCode::Numpad4 => {
                            impulse.impulse = Vec3::new(-BUMP, 0.0, 0.0);
                        }
                        KeyCode::ArrowRight => {
                            impulse.impulse = Vec3::new(BUMP, 0.0, 0.0);
                        }
                        KeyCode::Numpad6 => {
                            impulse.impulse = Vec3::new(BUMP, 0.0, 0.0);
                        }
                        KeyCode::ArrowUp => {
                            impulse.impulse = Vec3::new(0.0, 0.0, -BUMP);
                        }
                        KeyCode::Numpad8 => {
                            impulse.impulse = Vec3::new(0.0, 0.0, -BUMP);
                        }
                        KeyCode::ArrowDown => {
                            impulse.impulse = Vec3::new(0.0, 0.0, BUMP);
                        }
                        KeyCode::Numpad2 => {
                            impulse.impulse = Vec3::new(0.0, 0.0, BUMP);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
}


fn update_scoreboard(
    mut query: Query<&mut Text3d, With<Score>>,
    scoreboard: Res<ScoreBoard>,
    toy_query: Query<&Transform, With<Toy>>,
) {
    for mut text in query.iter_mut() {
        if scoreboard.round == 0 {
            *text = Text3d::new("Press enter\nto drop \na ball");
        } else {
            let remaining = toy_query.count();
            *text = Text3d::new(format!("Level: {}\nScore: {}\nTotal: {}\nRemaining: {}",
                              scoreboard.round, scoreboard.score, scoreboard.total, remaining));
        }
    }
}

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
)
{
    let mat = materials.add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        cull_mode: None,
        ..Default::default()
    });

    // Scoreboard text plus the board itself
    commands.spawn((
        Score{},
        Text3d::new("Starting..."),
        Text3dStyling {
            size: 64.,
            color: Srgba::BLACK,
            world_scale: Some(Vec2::splat(1.0)),
            layer_offset: 0.001,
            align: TextAlign::Center,
            ..Default::default()
        },
        NotShadowCaster,
        Mesh3d::default(),
        MeshMaterial3d(mat.clone()),
        Transform {
            translation: Vec3::new(-14.0, 5.5, 0.0),
            rotation: Quat::from_axis_angle(Vec3::Y, FRAC_PI_2),   // 90 degrees
            scale: Vec3::splat(1.5),
        },
    ));
    commands.spawn((
        RigidBody::Fixed,
        Friction::new(0.5),
        Restitution::new(0.1),
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.5, 7.0, 14.0)))),
        MeshMaterial3d(materials.add(SCOREBOARD_COLOR)),
        Collider::cuboid(0.25, 3.5, 7.0),
        Transform::from_xyz(-14.5, 5.0, 0.0),
    ));

    // commands.spawn((
    //     Text::new("Initializing..."),
    //     TextColor(TEXT_COLOR),
    //     TextFont {
    //         font_size: FontSize::Px(16.0),
    //         ..Default::default()
    //     },
    //     Node {
    //         position_type: PositionType::Absolute,
    //         top: px(12),
    //         left: px(12),
    //         ..default()
    //     },
    // ));

    // Spawn the Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraController::default(),
    ));
    // commands.spawn ((
    //         GlobalAmbientLight {
    //             color: Color::from(LIGHT_COLOR),
    //             brightness: 10_000_000.0,
    //             ..default()
    //         },
    //     ));

    // 2. Spawn a Light
    commands.spawn((
//        DirectionalLight::default(),
        PointLight {
            color: Color::from(LIGHT_COLOR),
            shadow_maps_enabled: true,
            intensity: 25_000_000.0,
            range: 80.0,
            radius: 1.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(0.0, 20.0, 10.0),
    ));


    // Surface plane
    commands.spawn((
        RigidBody::Fixed,
        Friction::new(0.5),
        Restitution::new(0.1),
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(25.0, 0.5, 20.0)))),
        MeshMaterial3d(materials.add(FLOOR_COLOR)),
        Collider::cuboid(12.5, 0.25, 10.0),
        Transform::from_xyz(0.0, -0.25, 0.0),
    ));
    // Barrier Left
    commands.spawn((
        RigidBody::Fixed,
        Friction::new(0.0),
        Restitution::new(0.1),
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(17.0, 0.5, 2.0)))),
        MeshMaterial3d(materials.add(BARRIER_COLOR)),
        Collider::cuboid(8.5, 0.25, 1.0),
        Transform::from_xyz(-4.0, 0.25, 0.0),
    ));

    // Barrier Right
    commands.spawn((
        RigidBody::Fixed,
        Friction::new(0.0),
        Restitution::new(0.1),
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(2.0, 0.5, 20.0)))),
        MeshMaterial3d(materials.add(BARRIER_COLOR)),
        Collider::cuboid(1.0, 0.25, 10.0),
        Transform::from_xyz(5.0, 0.25, 0.0),
    ));

    // Title
    let mat = materials.add(StandardMaterial {
        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        cull_mode: None,
        ..Default::default()
    });
    commands.spawn((
        Text3d::new("Holy Balls"),
        Text3dStyling {
            size: 64.,
            stroke: NonZero::new(10),
            color: Srgba::new(0.0, 0.0, 1.0, 1.),
            stroke_color: Srgba::BLACK,
            world_scale: Some(Vec2::splat(0.25)),
            layer_offset: 0.001,
            ..Default::default()
        },
        Mesh3d::default(),
        MeshMaterial3d(mat.clone()),
        Transform {
            translation: Vec3::new(1., 5., -10.0),
            rotation: Quat::from_axis_angle(Vec3::Y, 0.),
            scale: Vec3::splat(10.0),
        },
    ));
    // Add fence
    // commands.spawn( (
    //     Fence {},
    //     Friction::new(0.0),
    //     Restitution::new(0.1),
    //     WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cube_with_hole.glb#Fence"))),
    //     AsyncSceneCollider::default(),
    //     Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(1.0, 0.2, 1.0)),
    // ));
//    commands.trigger(CylinderEvent{});
    commands.write_message(DropToys{});
}
