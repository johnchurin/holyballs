
use bevy::audio::Volume;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::{FRAC_PI_2};
use std::num::NonZero;
use bevy::input::mouse::MouseMotion;
use bevy::light::NotShadowCaster;
use bevy::window::{PrimaryWindow, WindowMode};
use bevy_rapier3d::rapier::prelude::CollisionEventFlags;
use rand::RngExt;
use bevy_fontmesh::{FontMeshPlugin, JustifyText, TextAnchor, TextMesh, TextMeshStyle};
//use bevy_rich_text3d::{Text3d, Text3dPlugin, Text3dStyling, TextAlign, TextAtlas};
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
const DEVIL_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);
const ANGEL_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const SCOREBOARD_COLOR: Color = Color::srgb(0.5, 0.5, 0.0);
const TEXT_COLOR: Color = Color::srgb(0.5, 0.5, 0.5);
const CYLINDER_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
const _CYLINDER_HALF_HEIGHT: f32 = 2.0;
const AUTO_NEXT_LEVEL: bool = true;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Initialize the Rapier physics engine and the debug renderer
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
//        .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
        // .add_plugins(Text3dPlugin{
        //     default_atlas_dimension: (1024, 1024),
        //     load_system_fonts: true,
        //     ..Default::default()
        // })
//        .add_plugins(RichText3dPlugin) // Must be registered!
        .add_systems(Startup, setup_physics)
        .add_systems(Startup, setup_window)
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
            start_new_game.run_if(input_just_pressed(KeyCode::KeyG)),
            start_next_level.run_if(input_just_pressed(KeyCode::KeyN)),
            update_scoreboard.run_if(resource_changed::<ScoreBoard>),
            // mouse_look_system.run_if(|mouse: Res<ButtonInput<MouseButton>>| mouse.pressed(MouseButton::Left)),
        ))
        .add_systems(Update, (
            handle_impulse_message,
            handle_next_level,
            handle_sensor_events,
            handle_help_message,
            handle_sound,
            score_fallen_entities,
        )
    )
    .add_message::<ImpulseMessage>()
    .add_message::<HelpMessage>()
    .add_message::<NextLevel>()
    .add_message::<SoundMessage>()
    .run();
}
enum SoundType {
    Win,
    Lose,
    Bonus,
}
#[derive(Message)]
struct ImpulseMessage {
    entity: Entity,
    force: Vec3,
}

#[derive(Message)]
struct SoundMessage {
    sound_type: SoundType,
}
#[derive(Message)]
struct NextLevel {
}
enum HelpType {
    Score,
    Next,
}
#[derive(Message)]
struct HelpMessage {
    help_type: HelpType,
    text: String,
}
#[derive(Component)]
struct Score {
}
#[derive(Component)]
struct HelpWall {
}
#[derive(Component)]
struct ScoringWall {
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
struct Barrier {
}

#[derive(Component)]
struct SensorChild {
    next_color: Color,
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
    running: bool,
    score: i32,
    level: i32,
    total: i32,
    toys: usize,
    balls: usize,
}

impl ScoreBoard {
    fn new() -> Self {
        Self{running: false, score: 0, level: 0, total: 0, toys: 0, balls: 0}
    }
    fn hit(&mut self, incr: i32) {
        self.score += incr;
        self.total += incr;
    }
    fn use_a_ball(&mut self) {
        self.balls -= 1;
    }
    fn stop(&mut self) {
        println!("stopping");
        self.running = false;
    }
    fn start(&mut self) {
        println!("starting");
        self.running = true;
    }
    fn next_level(&mut self) {
        self.score = 0;
        self.level += 1;
        self.balls = 3;
    }
    fn reset(&mut self) {
        self.running = false;
        self.score = 0;
        self.level = 0;
        self.total = 0;
        self.balls = 0;
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
fn setup_window(
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    for mut window in &mut windows {
        window.set_maximized(true);
//        window.mode = WindowMode::Fullscreen{ 0: MonitorSelection::Current, 1: VideoModeSelection::Current };
        window.title = "Holy Balls".into();
    }
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
    ball_query: Query<(Entity, &BouncyBall), With<BouncyBall>>,
    mut toy_query: Query<(Entity, &mut Toy, &mut RigidBody, &mut PointValue, &MeshMaterial3d<StandardMaterial>), (With<Toy>, Without<SensorChild>)>,
    mut sensor_query: Query<(Entity, &ChildOf, &mut PointValue, &SensorChild), (With<SensorChild>, Without<Toy>)>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
                        let (child_entity, parent_entity, mut child_point_value, sensor_child) = sensor_query.get_mut(toy).unwrap();
                        if child_point_value.value != 0 {
                            let (_toy_entity, _toy_component, _rigid_body, mut parent_point_value, material_handle) = toy_query.get_mut(parent_entity.0).unwrap();
                            //                        println!("Parent entity: {:?} (pointvalue: {:?}), child: {:?} (pointvalue: {:?})",
                            //                                 parent_entity.0, parent_point_value.value, child_entity.entity(), child_point_value.value);
                            parent_point_value.value += child_point_value.value;
                            child_point_value.value = 0;
                            // But add the points to the parent of the toy
                            //                        println!("Sensor event");
                            commands.write_message(SoundMessage { sound_type: SoundType::Bonus });
                            // Change color (of parent) when bonus hits on a sensor child
                            if let Some(mut material) = materials.get_mut(material_handle) {
                                material.base_color = sensor_child.next_color;
                            }
                        }
                        return;
                    }
                    match bouncy_ball.status {
                        BouncyBallStatus::Live => {
                            for (toy_entity, mut toy_component, mut rigid_body,
                                _point_value, _material_handle) in toy_query.iter_mut() {
                                if toy_entity == toy && !toy_component.dynamic {
 //                                   println!("Make toy dynamic");
                                    *rigid_body = RigidBody::Dynamic;
                                    toy_component.dynamic = true;
                                    println!("Bump");
                                    commands.write_message(ImpulseMessage{entity: toy_entity, force:Vec3::new(2.0, 2.0, 0.1)});
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

fn score_fallen_entities(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ball_query: Query<(Entity, &BouncyBall, &mut Transform, &PointValue), (With<BouncyBall>, Without<Toy>)>,
    toy_query: Query<(Entity, &Toy, &Transform, &PointValue), (With<Toy>, Without<BouncyBall>)>,
    mut scoreboard: ResMut<ScoreBoard>,
) {
    scoreboard.toys = 0;
    // score and cleanup old toys and balls that are out of range
    for (entity, toy, transform, point_value) in toy_query.iter() {
//        scoreboard.remaining += 1;
        if transform.translation.y < -15.0 {
            commands.entity(entity).despawn();
            println!("Toy despawned {} points", point_value.value);
            if scoreboard.running {
                scoreboard.hit(point_value.value);
                if point_value.value > 0 {
                    commands.write_message(HelpMessage { help_type: HelpType::Score, text: format!("You scored {} points!", point_value.value) });
                } else {
                    commands.write_message(HelpMessage { help_type: HelpType::Score,text: format!("You lost {} points", -point_value.value) });
                }
                if point_value.value != 0 && scoreboard.running {
                    commands.write_message(
                        SoundMessage {
                            sound_type: if point_value.value < 0
                            { SoundType::Lose } else { SoundType::Win }
                        });
                }
            }
        } else {
            scoreboard.toys += 1;
        }
    }
    for (entity, ball, transform, point_value) in ball_query.iter() {
        if transform.translation.y < -15.0 {
            commands.entity(entity).despawn();
            println!("Ball despawned {} points", point_value.value);
            scoreboard.hit(point_value.value);
            if point_value.value != 0 {
                if point_value.value > 0 {
                    commands.write_message( HelpMessage{help_type: HelpType::Score, text: format!("You scored {} points!",point_value.value)});
                } else {
                    commands.write_message( HelpMessage{help_type: HelpType::Score, text: format!("You lost {} points", -point_value.value)});
                }
                // If our last, live ball, then game over.
                if scoreboard.balls == 0 {
                    match ball.status {
                        BouncyBallStatus::Live => {
                            commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Game Over. Press G to start new game.".to_string()});
                        }
                        _ => {}
                    }
                } else {
                    commands.write_message( HelpMessage{help_type: HelpType::Score,
                        text: format!("You scored {} points. Press Enter for another ball", point_value.value)});
                }

                commands.write_message(
                    SoundMessage {
                        sound_type: if point_value.value < 0
                        { SoundType::Lose } else { SoundType::Win }
                    });
            }
        }
    }

    // Don't make a sound for zero point value
    if scoreboard.running && scoreboard.toys == 0 {
        // Round is no longer running
        scoreboard.stop();
        scoreboard.hit(100);
        commands.write_message( HelpMessage{help_type: HelpType::Score, text: "100 points for clearing this level.".to_string()});
        let mut text = format!("Press N to start level {}.", scoreboard.level+1);
        if scoreboard.balls > 1 {
            text += "\nFor extra points drop another ball and push the dead ball(s) off the edge, too.";
        }
        commands.write_message( HelpMessage{help_type: HelpType::Next, text});
        commands.write_message(SoundMessage{sound_type: SoundType::Win});
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
fn handle_help_message (
    mut messages: MessageReader<HelpMessage>,
    mut help_query: Query<&mut TextMesh, (With<HelpWall>, Without<ScoringWall>)>,
    mut score_query: Query<&mut TextMesh, (With<ScoringWall>, Without<HelpWall>)>,
    mut commands: Commands,
) {
    for message in messages.read() {
        match message.help_type {
            HelpType::Score => {
                for mut text_mesh in score_query.iter_mut() {
                    text_mesh.text = message.text.clone();
                }
            }
            HelpType::Next => {
                for mut text_mesh in help_query.iter_mut() {
                    text_mesh.text = message.text.clone();
                }
            }
        }
    }
}
fn handle_impulse_message (
    mut messages: MessageReader<ImpulseMessage>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut ExternalImpulse), With<Toy>>,
) {
    for message in messages.read() {
        for (entity, mut external_impulse) in query.iter_mut() {
            if entity.entity() ==  message.entity.entity() {
                external_impulse.impulse = message.force;
            }
        }
    }
}

fn handle_sound(
    mut messages: MessageReader<SoundMessage>,
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
            SoundType::Bonus => {
                commands.spawn((
                    AudioPlayer::new(asset_server.load("audio/tinkle.ogg")),
                    PlaybackSettings::ONCE,
                ));
            }
        }
    }
}
fn random_location() -> Vec3 {
    let mut rng = rand::rng();
    Vec3::new(rng.random_range(-10.0..10.0),
              10.0 + rng.random_range(0.0..10.0),
              rng.random_range(-9.0..9.0))
}
fn handle_next_level(
    mut messages: MessageReader<NextLevel>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scoreboard: ResMut<ScoreBoard>,
    mut old_balls: Query<Entity, With<BouncyBall>>,
    mut old_toys: Query<Entity, With<Toy>>,
    mut old_barriers: Query<Entity, With<Barrier>>,
) {
    for _event in messages.read() {
        for entity in old_balls.iter_mut() {
            commands.entity(entity).despawn();
        }
        for entity in old_toys.iter_mut() {
            commands.entity(entity).despawn();
        }
        for entity in old_barriers.iter_mut() {
            commands.entity(entity).despawn();
        }
        scoreboard.next_level();
        if scoreboard.total == 0 {
            commands.write_message( HelpMessage{help_type: HelpType::Score, text: "No score yet".to_string()});
        } else if scoreboard.score == 0 {
            commands.write_message( HelpMessage{help_type: HelpType::Score, text: format!("No score for level {}",scoreboard.level)});
        }
        commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Press Enter to drop a ball".to_string()});
//        println!("Level: {}", scoreboard.level);
        scoreboard.start();
        let mut rng = rand::rng();
        if scoreboard.level > 1 {
            // Barrier Left
            commands.spawn((
                Barrier {},
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
                Barrier {},
                RigidBody::Fixed,
                Friction::new(0.0),
                Restitution::new(0.1),
                Mesh3d(meshes.add(Mesh::from(Cuboid::new(2.0, 0.5, 20.0)))),
                MeshMaterial3d(materials.add(BARRIER_COLOR)),
                Collider::cuboid(1.0, 0.25, 10.0),
                Transform::from_xyz(5.0, 0.25, 0.0),
            ));
        }
        // A Target
        if scoreboard.level > 2 {
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
                Transform::from_xyz(-14.0, 3.0, 5.0).with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
            ));
            commands.spawn((
                Toy { dynamic: false },
                RigidBody::Fixed,
                PointValue { value: 45 },
                ExternalImpulse::default(), // For when this becomes dynamic
                Friction::new(0.1),
                Restitution::new(0.1),
                Collider::cuboid(0.1, 1.0, 1.0),
                Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.2, 1.5, 1.5)))),
                MeshMaterial3d(materials.add(TARGET_COLOR)),
                Transform::from_xyz(-14.0, 7.0, -5.0),
            ));
        }
        // // Local cones
        // for _n in 0..2 {
        //     commands.spawn((
        //         Toy { dynamic: true },
        //         RigidBody::Dynamic,
        //         Friction::new(0.9),
        //         Restitution::new(0.1),
        //         Mesh3d(meshes.add(Mesh::from(Cone::new(0.75, 2.0)))),
        //         MeshMaterial3d(materials.add(CONE_COLOR)),
        //         ExternalImpulse::default(),
        //         PointValue { value: 15 },
        //         // Lower the Damping for a more advanced game
        //         // Damping {
        //         //     linear_damping: 0.2,
        //         //     angular_damping: 0.2,
        //         // },
        //         Collider::cone(1.0, 0.75),
        //         Transform::from_xyz(rng.random_range(-12.0..12.0),
        //                             10.0 + rng.random_range(0.0..10.0),
        //                             rng.random_range(-9.0..9.0)),
        //         ExternalForce::default(),
        //     ));
        // }
        // // Local disks
        // for _n in 0..4 {
        //     commands.spawn((
        //         RigidBody::Dynamic,
        //         Toy { dynamic: true },
        //         Friction::new(0.2),
        //         Restitution::new(0.1),
        //         Mesh3d(meshes.add(Mesh::from(Cylinder::new(0.75, 0.6)))),
        //         MeshMaterial3d(materials.add(DISK_COLOR)),
        //         ExternalImpulse::default(),
        //         PointValue { value: 15 },
        //         // Lower the Damping for a more advanced game
        //         // Damping {
        //         //     linear_damping: 0.2,
        //         //     angular_damping: 0.2,
        //         // },
        //         Collider::cylinder(0.3, 0.75),
        //         Transform::from_xyz(rng.random_range(-12.0..12.0),
        //                             10.0 + rng.random_range(0.0..10.0),
        //                             rng.random_range(-9.0..9.0)),
        //         ExternalForce::default(),
        //     ));
        // }
        // commands.spawn((
        //     RigidBody::Dynamic,
        //     Toy { dynamic: true },
        //     Friction::new(0.2),
        //     Restitution::new(0.1),
        //     Mesh3d(meshes.add(Mesh::from(Cylinder::new(0.75, 0.6)))),
        //     MeshMaterial3d(materials.add(DISK_COLOR)),
        //     ExternalImpulse::default(),
        //     PointValue { value: 15 },
        //     // Lower the Damping for a more advanced game
        //     // Damping {
        //     //     linear_damping: 0.2,
        //     //     angular_damping: 0.2,
        //     // },
        //     Collider::cylinder(0.3, 0.75),
        //     Transform::from_xyz(rng.random_range(-12.0..12.0),
        //                         10.0 + rng.random_range(0.0..10.0),
        //                         rng.random_range(-9.0..9.0)),
        //     ExternalForce::default(),
        // ));
        // // Dip
        // commands.spawn((
        //     RigidBody::Dynamic,
        //     Toy { dynamic: true },
        //     ExternalImpulse::default(),
        //     Friction::new(0.0),
        //     Restitution::new(0.1),
        //     PointValue { value: 10 },
        //     WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/dip.glb#collection"))),
        //     AsyncSceneCollider::default(),
        //     Transform::from_xyz(-9.0, 8.0, 3.0).with_scale(Vec3::splat(0.4)),
        // )).with_children(|parent| {
        //     parent.spawn((
        //         SensorChild {},
        //         Collider::ball(0.1),
        //         Sensor,
        //         PointValue { value: 25 },
        //         ActiveEvents::COLLISION_EVENTS,
        //         Transform::from_xyz(0.0, 0.2, 0.0),
        //     ));
        // });
        //
        // Boxes
        for _n in 0..4 {
            commands.spawn((
                RigidBody::Dynamic,
                Toy { dynamic: true },
                Friction::new(0.2),
                Restitution::new(0.1),
                Mesh3d(meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: BOX_COLOR,
//                    alpha_mode: AlphaMode::Blend,
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
                Transform::from_translation(random_location()),
                ExternalForce::default(),
            ));
        };
        // Transparent boxes
        if scoreboard.level > 4 {
            for _n in 0..4 {
                commands.spawn((
                    RigidBody::Dynamic,
                    Toy { dynamic: true },
                    Friction::new(0.2),
                    Restitution::new(0.1),
                    Mesh3d(meshes.add(Mesh::from(Cuboid::new(1.0, 1.0, 1.0)))),
                    MeshMaterial3d(materials.add(StandardMaterial {
                        base_color: BOX_COLOR_TRANSPARENT,
                        alpha_mode: AlphaMode::Blend,
                        ..default()
                    })),
                    NotShadowCaster,
                    ExternalImpulse::default(),
                    PointValue { value: 20 },
                    // Lower the Damping for a more advanced game
                    // Damping {
                    //     linear_damping: 0.2,
                    //     angular_damping: 0.2,
                    // },
                    Collider::cuboid(0.5, 0.5, 0.5),
                    Transform::from_translation(random_location()),
                    ExternalForce::default(),
                ));
            };
        }
        // // Doughnut (torus)
        // commands.spawn((
        //     RigidBody::Dynamic,
        //     Toy { dynamic: true },
        //     Friction::new(0.0),
        //     Restitution::new(0.1),
        //     ExternalImpulse::default(),
        //     PointValue { value: 10 },
        //     ColliderMassProperties::Density(0.25),
        //     WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/doughnut.glb#collection"))),
        //     AsyncSceneCollider::default(),
        //     Transform::from_xyz(10.0, 12.0, 7.0).with_scale(Vec3::splat(1.0)),
        // )).with_children(|parent| {
        //     parent.spawn((
        //         SensorChild {},
        //         Collider::ball(0.1),
        //         PointValue { value: 20 },
        //         Sensor,
        //         ActiveEvents::COLLISION_EVENTS,
        //         Transform::from_xyz(0.0, 0.2, 0.0),
        //     ));
        // });
        //
        // // Cylinder
        // for _n in 0..6 {
        //     commands.spawn((
        //         RigidBody::Dynamic,
        //         Toy { dynamic: true },
        //         Friction::new(0.8),
        //         Restitution::new(0.1),
        //         ExternalImpulse::default(),
        //         PointValue { value: 5 },
        //         WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cylinder.glb#collection"))),
        //         AsyncSceneCollider::default(),
        //         Transform::from_xyz(rng.random_range(-12.0..12.0), 4.0, rng.random_range(-9.0..9.0)).with_scale(Vec3::splat(0.75)),
        //     ));
        // }
        // // Devil Disk
        if scoreboard.level > 5 {
            commands.spawn((
                RigidBody::Dynamic,
                Toy { dynamic: true },
                Friction::new(0.2),
                Restitution::new(0.1),
                Mesh3d(meshes.add(Mesh::from(Cylinder::new(0.75, 0.6)))),
                MeshMaterial3d(materials.add(DEVIL_COLOR)),
                ExternalImpulse::default(),
                PointValue { value: -50 },
                // Lower the Damping for a more advanced game
                // Damping {
                //     linear_damping: 0.2,
                //     angular_damping: 0.2,
                // },
                Collider::cylinder(0.3, 0.75),
                Transform::from_translation(random_location()),
                ExternalForce::default(),
                )).with_children(|parent| {
                    parent.spawn((
                        SensorChild {next_color: ANGEL_COLOR},
                        Collider::ball(0.2),
                        Sensor,
                        PointValue { value: 100 },
                        ActiveEvents::COLLISION_EVENTS,
                        Transform::from_xyz(0.0, 0.6, 0.0),
                    ));
                });
//            ));
        }
        // Bumpy sphere
        if scoreboard.level > 3 {
            commands.spawn((
                RigidBody::Dynamic,
                Toy { dynamic: true },
                Friction::new(0.0),
                Restitution::new(0.1),
                ExternalImpulse::default(),
                PointValue { value: 5 },
                WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/bumpy.glb#collection"))),
                AsyncSceneCollider::default(),
                Transform::from_translation(random_location()),
            ));
        }
    }
}
fn start_new_game(
    mut scoreboard: ResMut<ScoreBoard>,
    mut commands: Commands,
) {
    scoreboard.reset();
    println!("Sending next level from start_new_game");
    commands.write_message(NextLevel {});
    commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Press the N key to start the first level".to_string()});
}
fn start_next_level(
    mut scoreboard: ResMut<ScoreBoard>,
//    mut old_balls: Query<Entity, With<BouncyBall>>,
    mut commands: Commands,
) {
    // for entry in old_balls.iter() {
    //     println!("Entity: {:?}", entry);
    // }
    println!("Sending next level from start_next_Level");
    commands.write_message(NextLevel {});
}
fn drop_a_ball(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(&mut BouncyBall, &mut PointValue, &MeshMaterial3d<StandardMaterial>), With<BouncyBall>>,
    mut scoreboard: ResMut<ScoreBoard>,
) {
    // Deduct 1 ball from count
    if scoreboard.balls == 0 {
        if scoreboard.running {
            commands.write_message(HelpMessage { help_type: HelpType::Score, text: "You have no balls left".to_string() });
        } else {
            commands.write_message(HelpMessage { help_type: HelpType::Next, text: "First, press G to start a game".to_string() });
        }
//        println!("You have no balls");
        return;
    }
    if scoreboard.level == 1 {
        commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Use arrow keys to move the ball around\nand push toys off the edge".to_string()});
    } else if scoreboard.level == 2 {
        commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Use space bar to bounce the ball.".to_string()});
    } else if scoreboard.level == 3 {
        commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Hit the toys on the scoreboard, too".to_string()});
    } else {
        commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Go for it.".to_string()});
    }
    scoreboard.use_a_ball();
    // Make any live balls dead, usually only one
    for (mut bouncyball, mut point_value, material_handle) in query.iter_mut() {
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
    commands.spawn((
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
    ));
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
    mut scoreboard_query: Query<&mut TextMesh, With<Score>>,
    scoreboard: Res<ScoreBoard>,
) {
    for mut text in scoreboard_query.iter_mut() {
        text.text = format!("Game Level: {}\nScore this level: {}\nTotal Score: {}\nToys Left: {}\nBalls Left: {}",
                          scoreboard.level,
                          scoreboard.score, scoreboard.total,
                            scoreboard.toys, scoreboard.balls);
        };
}

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,

)
{
    let font = asset_server.load("fonts/FiraMono-Medium.ttf");

    commands.spawn((
        Score{},
        TextMesh {
            text: "Starting".to_string(),
            font: font.clone(),
            style: TextMeshStyle {
                depth: 0.1,
                subdivision: 8,
                anchor: TextAnchor::Center,
                justify: JustifyText::Center,
                ..default()
            },
        },
        NotShadowCaster,
        Mesh3d::default(),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.3, 0.8), // Blueish metallic
            metallic: 0.8,             // Slightly less metallic to show some base color
            perceptual_roughness: 0.3, // Rougher to catch more light highlights
            reflectance: 0.8,
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        Transform {
            translation: Vec3::new(-14.0, 5.0, 0.0),
            rotation: Quat::from_axis_angle(Vec3::Y, FRAC_PI_2),   // 90 degrees
            scale: Vec3::splat(0.9),
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
//    let test = Text3d::parse_raw("{font-family:Arial}{color:red}I got nothing");

    // Help wall
    commands.spawn((
        HelpWall{},
        TextMesh {
            text: "Welcome!".to_string(),
            font: font.clone(),
            style: TextMeshStyle {
                depth: 0.08,
                subdivision: 8,
                anchor: TextAnchor::Center,
                justify: JustifyText::Center,
                ..default()
            },
        },
        NotShadowCaster,
        Mesh3d::default(),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.3, 0.8), // Blueish metallic
            metallic: 0.8,             // Slightly less metallic to show some base color
            perceptual_roughness: 0.3, // Rougher to catch more light highlights
            reflectance: 0.8,
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        Transform {
            translation: Vec3::new(-0.0, -1.0, 12.0),
            rotation: Quat::from_axis_angle(Vec3::Y, 0.0),
            scale: Vec3::splat(0.6),
        },
    ));
    // Scoring wall
    commands.spawn((
        ScoringWall{},
        TextMesh {
            text: "No score yet".to_string(),
            font: font.clone(),
            style: TextMeshStyle {
                depth: 0.08,
                subdivision: 8,
                anchor: TextAnchor::Center,
                justify: JustifyText::Center,
                ..default()
            },
        },
        NotShadowCaster,
        Mesh3d::default(),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.3, 0.8),
            metallic: 0.8,
            perceptual_roughness: 0.3,
            reflectance: 0.8,
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        Transform {
            translation: Vec3::new(0.0, 2.0, -10.0),
            rotation: Quat::from_axis_angle(Vec3::Y, 0.0),
            scale: Vec3::splat(0.6),
        },
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
    // Title
    let mat = materials.add(StandardMaterial {
//        base_color_texture: Some(TextAtlas::DEFAULT_IMAGE.clone()),
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        cull_mode: None,
        ..Default::default()
    });
    commands.spawn((
        TextMesh {
            text: "Holy Balls".to_string(),
            font: font.clone(),
            style: TextMeshStyle {
                depth: 0.4,
                subdivision: 8,
                anchor: TextAnchor::Center,
                justify: JustifyText::Center,
                ..default()
            },
        },
        Mesh3d::default(),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: LIVE_BALL,
            metallic: 0.8,             // Slightly less metallic to show some base color
            perceptual_roughness: 0.3, // Rougher to catch more light highlights
            reflectance: 0.8,
            double_sided: true,
            cull_mode: None,
            ..default()
        })),
        Transform {
            translation: Vec3::new(0., 5., -10.0),
            rotation: Quat::from_axis_angle(Vec3::Y, 0.),
            scale: Vec3::splat(4.0),
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
//    commands.write_message(NextLevel {});
    commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Press the G key to start a new game".to_string()});
}
