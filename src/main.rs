
use bevy::audio::Volume;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::{FRAC_PI_2};
use bevy::light::NotShadowCaster;
use bevy::log::Level;
use bevy::window::{PrimaryWindow};
use bevy_rapier3d::rapier::prelude::CollisionEventFlags;
use rand::RngExt;
use bevy_fontmesh::{FontMeshPlugin, JustifyText, TextAnchor, TextMesh, TextMeshStyle};
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
const BLACK_DISK_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);
const WHITE_DISK_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
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
        .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
        // .add_plugins(Text3dPlugin{
        //     default_atlas_dimension: (1024, 1024),
        //     load_system_fonts: true,
        //     ..Default::default()
        // })
//        .add_plugins(RichText3dPlugin) // Must be registered!
        .add_systems(Startup, setup_configuration)
        .add_systems(Startup, setup_physics)
        .add_systems(Startup, setup_window)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(ScoreBoard::new())
        .insert_resource(GlobalVolume::new(Volume::Linear(0.25)))
        .insert_resource(ScoringHelpTimer::new(8.0))
        .insert_resource(Configuration::new())
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
            clear_scoring_text,
        ))
        .add_systems(Update, (
            handle_point_value_message,
            handle_activate_game,
            handle_impulse_message,
            handle_next_level,
            handle_sensor_events,
            handle_help_message,
            handle_sound,
            handle_asset_color_propagation,
            score_fallen_entities,
        ))
    .add_message::<PointValueMessage>()
    .add_message::<PropagateAssetColorMessage>()
    .add_message::<ActivateGameMessage>()
    .add_message::<ImpulseMessage>()
    .add_message::<HelpMessage>()
    .add_message::<NextLevel>()
    .add_message::<SoundMessage>()
    .run();
}
#[derive(Resource)]
struct ScoringHelpTimer {
    entity: Option<Entity>,
    start: f32,
    duration: f32,
    active: bool,
}
#[derive(Default, Clone, Debug)]
struct Toy {
    barriers: i32,
    blocks: i32,
    disks: i32,
    cones: i32,
    blacks: i32,
    dips: i32,
    bumpys: i32,
    targets: i32,
    ghosts: i32,
    lifesavers: i32,
    cylinders: i32,
}
#[derive(Resource)]
struct Configuration {
    levels: Vec<Toy>,
}
impl Configuration {
    fn new() -> Self {
        Self{ levels: Vec::new() }
    }

    fn add(&mut self, level: Toy) -> &mut Self {
        self.levels.push(level);
        self
    }

    fn get_toys(&self, level: i32) -> &Toy {
        // Level is 1 origin, levels Vec is zero origin, so level-1)
        if level > self.levels.len() as i32 {
            // If beyond the end, just return the last toy level
            self.levels.get(self.levels.len() - 1usize).unwrap()
        } else {
            self.levels.get(level as usize - 1).unwrap()
        }
    }
}
impl ScoringHelpTimer {
    fn new(duration: f32) -> Self {
        Self { entity: None, active: false, duration, start: 0.0 }
    }
    fn start(&mut self, entity:Entity, start: f32) {
        self.entity = Some(entity);
        self.active = true;
        self.start = start;
    }
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
struct ActivateGameMessage {
}
#[derive(Message, Debug)]
struct PropagateAssetColorMessage{
    entity: Entity,
    color: Color,
}
#[derive(Message, Debug)]
struct PointValueMessage{
    entity: Entity,
    value: i32,
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

#[derive(Component, Clone, Copy, Debug)]
struct BouncyBall {
    live: bool,
}
#[derive(Component)]
struct Barrier {
}

#[derive(Component, Debug)]
struct SensorChild {
    next_color: Color,
}

#[derive(Component)]
struct PointValue {
    value: i32,
}

#[derive(Component)]
struct ToyType {
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
        println!("stopping game");
        self.running = false;
    }
    fn start(&mut self) {
        println!("starting game");
        self.running = true;
    }
    fn next_level(&mut self) {
        self.score = 0;
        self.level += 1;
        self.balls = 3;
    }
    fn reset(&mut self) {
        println!("reset scoreboard");
        self.running = false;
        self.score = 0;
        self.level = 0;
        self.total = 0;
        self.balls = 0;
    }
}
#[derive(Component)]
struct CameraController {
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

fn setup_configuration(
    mut configuration: ResMut<Configuration>,
) {
    configuration.add(Toy{blocks:1,..Toy::default()});
    configuration.add(Toy{blocks:2, disks: 2,..Toy::default()});
    configuration.add(Toy{barriers: 1, blocks:2, disks: 2,..Toy::default()});
    configuration.add(Toy{barriers: 2, blocks:2, blacks: 1,..Toy::default()});
    configuration.add(Toy{barriers: 2, blocks:2, blacks: 1, dips:2,..Toy::default()});
    configuration.add(Toy{barriers: 2, targets:2, ..Toy::default()});
    configuration.add(Toy{barriers: 2, disks: 2, cylinders:3, ..Toy::default()});
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
fn clear_scoring_text(
    time: Res<Time>,
    mut timer: ResMut<ScoringHelpTimer>,
    mut query: Query<&mut Visibility>,
) {
    if timer.active {
        if timer.entity.is_some() && time.elapsed_secs() > timer.start+timer.duration {
            if let Ok(mut visibility) = query.get_mut(timer.entity.unwrap()) {
                *visibility = Visibility::Hidden;
            }
            timer.active = false;
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
fn handle_sensor_events(
    mut messages: ResMut<Messages<CollisionEvent>>,
    ball_query: Query<(Entity, &BouncyBall), With<BouncyBall>>,
    mut toy_query: Query<Entity, (With<ToyType>, Without<SensorChild>)>,
    mut sensor_query: Query<(Entity, &ChildOf, &mut PointValue, &SensorChild), (With<SensorChild>, Without<ToyType>)>,
    mut rigid_query: Query<(Entity, &mut ToyType, &mut RigidBody), (With<ToyType>, Without<SensorChild>)>,
    mut commands: Commands,
) {
    for event in messages.drain() {
        match event {
            CollisionEvent::Started(entity1, entity2, flags) => {
                // Separate handling for live ball hitting a sensor and live ball hitting a fixed rigid body toy
                if flags.contains(CollisionEventFlags::SENSOR) {
                    // Focus on the (only) live ball which is the only entity that can collide with another entity (including dead balls)
                    for (ball_entity, bouncy_ball) in ball_query.iter() {
                        if !bouncy_ball.live {continue} // live balls only
                        // Sort out which of the pair is the sensor (the other one is the ball)
                        let sensor = if ball_entity == entity2 { entity1 } else if ball_entity == entity1 { entity2 } else { continue };
                        // The ball collides with the SensorChild, not its parent toy
                        for (sensor_entity, parent, mut child_point_value, sensor_child) in sensor_query.iter_mut() {
                            // Now get the toy so we can add the points in
                            if sensor == sensor_entity {
                                if let Ok(toy_entity) = toy_query.get_mut(parent.0) {
                                    println!("parent.0: {:?}, toy: {:?}", parent.0.entity(), toy_entity);
                                    if child_point_value.value != 0 {
                                        commands.write_message(PointValueMessage { entity: toy_entity, value: child_point_value.value });
                                        commands.write_message(HelpMessage { help_type: HelpType::Score, text: format!("Bonus earns an extra {} points", child_point_value.value) });
                                        child_point_value.value = 0;
                                        commands.write_message(SoundMessage { sound_type: SoundType::Bonus });
                                        // Change color (of parent and descendents) when bonus hits on a sensor child
                                        commands.write_message(PropagateAssetColorMessage { entity: toy_entity, color: sensor_child.next_color });
                                    }
                                }
                            }
                        }
                    }
                } else {
                    // Focus on the live ball colliding with a fixed rigid body toy
                    for (ball_entity, bouncy_ball) in ball_query.iter() {
                        // Live balls only
                        if !bouncy_ball.live {continue}
                        // Only want live balls hitting SensorChilds for this type of collision
                        let toy = if ball_entity == entity2 { entity1 } else if ball_entity == entity1 { entity2 } else { continue };
                        for (toy_entity, mut toy_component, mut rigid_body) in rigid_query.iter_mut() {
                            if toy_entity == toy && !toy_component.dynamic {
                                //                                   println!("Make toy dynamic");
                                *rigid_body = RigidBody::Dynamic;
                                toy_component.dynamic = true;
//                                println!("Bump");
                                commands.write_message(ImpulseMessage { entity: toy_entity, force: Vec3::new(2.0, 2.0, 0.1) });
                            }
                        }
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
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    ball_query: Query<(Entity, &BouncyBall, &mut Transform, &PointValue), (With<BouncyBall>, Without<ToyType>)>,
    toy_query: Query<(Entity, &Transform, &PointValue), (With<ToyType>, Without<BouncyBall>)>,
    mut scoreboard: ResMut<ScoreBoard>,
) {
    scoreboard.toys = 0;
    // score and cleanup old toys and balls that are out of range
    for (entity, transform, point_value) in toy_query.iter() {
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
    if scoreboard.running && scoreboard.toys == 0 {
        // Round is no longer running
        scoreboard.stop();
        println!("{} toys found", scoreboard.toys);
        scoreboard.hit(100);
        commands.write_message( HelpMessage{help_type: HelpType::Score,
                text: "100 points for clearing this level".to_string()});
        let text = format!("Press N to start level {}", scoreboard.level+1);
        commands.write_message( HelpMessage{help_type: HelpType::Next, text});
        commands.write_message(SoundMessage{sound_type: SoundType::Win});
        return;
    }
    // Look for fallen balls
    for (entity, ball, transform, point_value) in ball_query.iter() {
        if transform.translation.y < -15.0 {
            commands.entity(entity).despawn();
            println!("Ball despawned {} points", point_value.value);
            if scoreboard.running {
                scoreboard.hit(point_value.value);
                if point_value.value != 0 {
                    if point_value.value > 0 {
                        commands.write_message( HelpMessage{help_type: HelpType::Score, text: format!("You scored {} points",point_value.value)});
                    } else {
                        commands.write_message( HelpMessage{help_type: HelpType::Score, text: format!("You lost {} points", -point_value.value)});
                    }
                    // If this is our last, live ball, then game over.
                    if scoreboard.balls == 0 {
                        if ball.live {
                            commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Game Over. Press G to start new game".to_string()});
                        }
                    } else {
                        if point_value.value < 0 {
                            commands.write_message( HelpMessage{help_type: HelpType::Score,
                                text: format!("That cost you {} points. Press Enter for another ball", point_value.value)});
                        } else {
                            commands.write_message( HelpMessage{help_type: HelpType::Score,
                                text: format!("You scored {} points. Press Enter for another ball", point_value.value)});
                        }
                    }

                    commands.write_message(
                        SoundMessage {
                            sound_type: if point_value.value < 0
                            { SoundType::Lose } else { SoundType::Win }
                        });
                }

            }
        }
    }
}

fn handle_help_message (
    mut messages: MessageReader<HelpMessage>,
    mut help_query: Query<&mut TextMesh, (With<HelpWall>, Without<ScoringWall>)>,
    mut score_query: Query<(Entity, &mut TextMesh, &mut Visibility), (With<ScoringWall>, Without<HelpWall>)>,
    mut countdown: ResMut<ScoringHelpTimer>,
    time: Res<Time>,
) {
    for message in messages.read() {
        match message.help_type {
            HelpType::Score => {
                for (entity, mut text_mesh, mut visibility) in score_query.iter_mut() {
                    *visibility = Visibility::Visible { };
                    if countdown.active {
                        text_mesh.text += "\nand ";
                        text_mesh.text += message.text.as_str();
                    } else {
                        text_mesh.text = message.text.clone();
                    }
                    countdown.start(entity, time.elapsed_secs());
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
// Add points to a toy
fn handle_point_value_message (
    mut messages: MessageReader<PointValueMessage>,
    mut query: Query<(&mut PointValue), With<ToyType>>,
) {
    for message in messages.read() {
        if let Ok(mut point_value) = query.get_mut(message.entity) {
            point_value.value += message.value;
        }
    }
}
fn handle_impulse_message (
    mut messages: MessageReader<ImpulseMessage>,
    mut query: Query<(Entity, &mut ExternalImpulse), With<ToyType>>,
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

fn handle_activate_game(
    mut messages: MessageReader<ActivateGameMessage>,
//    mut commands: Commands,
    mut scoreboard: ResMut<ScoreBoard>,
) {
    for _event in messages.read() {
        scoreboard.start();
    }
}
fn handle_asset_color_propagation(
    mut messages: MessageReader<PropagateAssetColorMessage>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
    children_query: Query<&Children>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands
) {
    for message in messages.read() {
//        println!("Color message: {:?}", message);
        // Update material at this level
        if let Ok(material_handle) = material_query.get(message.entity) {
//            println!("Get material for: {:?}", message);
            if let Some(material) = materials.get(material_handle) {
                // 4. Unique copy so we don't accidentally color every object red
                let mut unique_material = material.clone();
                unique_material.base_color = message.color;
                // 5. Overwrite the child's component with the unique material
                commands.entity(message.entity).insert(MeshMaterial3d(materials.add(unique_material)));
            }
        }
        // Then recurse to the children, if any
        if let Ok (children) = children_query.get(message.entity) {
            for child in children.iter() {
//                println!("Child Entity: {}", child);
                // And for good measure, continue down to other descendents
                commands.write_message(PropagateAssetColorMessage{ entity: child, color: message.color});
            }
        }
    }
}
fn create_barriers(
    n: i32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    //    asset_server: AssetServer,
) {
    if n > 0 {
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
    }
    if n > 1 {
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
}
fn create_blocks(
    n: i32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    //    asset_server: AssetServer,
) {
    // Boxes
    for _n in 0..n {
        commands.spawn((
            RigidBody::Dynamic,
            ToyType { dynamic: true },
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
}
fn create_disks(
    n: i32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    //    asset_server: AssetServer,
) {
    for _n in 0..n {
        commands.spawn((
            RigidBody::Dynamic,
            ToyType { dynamic: true },
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
            Transform::from_translation(random_location()),
            ExternalForce::default(),
        ));
    }

}

fn create_cones(
    n: i32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    //    asset_server: AssetServer,
){
    for _n in 0..n {
        commands.spawn((
            ToyType { dynamic: true },
            RigidBody::Dynamic,
            Friction::new(0.1),
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
            Transform::from_translation(random_location()),
            ExternalForce::default(),
        ));
    }
}

fn create_blacks(
    n: i32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
//    asset_server: AssetServer,
) {
    for _n in 0..n {
        commands.spawn((
            RigidBody::Dynamic,
            ToyType { dynamic: true },
            Friction::new(0.2),
            Restitution::new(0.1),
            Mesh3d(meshes.add(Mesh::from(Cylinder::new(0.75, 0.6)))),
            MeshMaterial3d(materials.add(BLACK_DISK_COLOR)),
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
                SensorChild {next_color: WHITE_DISK_COLOR },
                Collider::ball(0.2),
                Sensor,
                PointValue { value: 20 },
                ActiveEvents::COLLISION_EVENTS,
                Transform::from_xyz(0.0, 0.6, 0.0),
            ));
            parent.spawn((
                SensorChild {next_color: WHITE_DISK_COLOR },
                Collider::ball(0.7),
                Sensor,
                PointValue { value: 20 },
                ActiveEvents::COLLISION_EVENTS,
                Transform::from_xyz(0.0, -0.6, 0.0),
            ));
        });
    }
}
fn create_dips(
    n: i32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &Res<AssetServer>,
){
    for _n in 0..n {
        commands.spawn((
            RigidBody::Dynamic,
            ToyType { dynamic: true },
            ExternalImpulse::default(),
            Friction::new(0.1),
            Restitution::new(0.1),
            ColliderMassProperties::Density(0.0),
            // Lower the Damping for a more advanced game
            Damping {
                linear_damping: 0.2,
                angular_damping: 0.2,
            },
            PointValue { value: 15 },
            MeshMaterial3d(materials.add(BLACK_DISK_COLOR)),
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/dip.glb#collection"))),
            AsyncSceneCollider::default(),
            Transform::from_translation(random_location()).with_scale(Vec3::splat(0.5)).with_rotation(Quat::from_axis_angle(Vec3::Z, FRAC_PI_2*0.8)),
        )).with_children(|parent| {
            parent.spawn((
                SensorChild {next_color: WHITE_DISK_COLOR},
                Collider::ball(0.1),
                Sensor,
                PointValue { value: 25 },
                ActiveEvents::COLLISION_EVENTS,
                Transform::from_xyz(0.0, 0.2, 0.0),
            ));
            parent.spawn((
                SensorChild {next_color: WHITE_DISK_COLOR },
                Collider::ball(0.7),
                Sensor,
                PointValue { value: 20 },
                ActiveEvents::COLLISION_EVENTS,
                Transform::from_xyz(0.0, -0.1, 0.0),
            ));
        });
    }
}
fn create_bumpys (
    n: i32,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
){
    for _n in 0..n {
        commands.spawn((
            RigidBody::Dynamic,
            ToyType { dynamic: true },
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
fn create_targets(
    n: i32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
){
    if n > 0 {
        commands.spawn((
            ToyType { dynamic: false },
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
    }
    if n > 1 {
    commands.spawn((
            ToyType { dynamic: false },
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
}

fn create_ghosts(
    n: i32,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for _n in 0..n {
        commands.spawn((
            RigidBody::Dynamic,
            ToyType { dynamic: true },
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
fn create_lifesavers (
    n: i32,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    for _n in 0..n {
        commands.spawn((
            RigidBody::Dynamic,
            ToyType { dynamic: true },
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
                SensorChild {next_color: WHITE_DISK_COLOR},
                Collider::ball(0.1),
                PointValue { value: 20 },
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                Transform::from_xyz(0.0, 0.2, 0.0),
            ));
        });
    }
}
fn create_cylinders(
    n: i32,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    for _n in 0..n {
        commands.spawn((
            RigidBody::Dynamic,
            ToyType { dynamic: true },
            Friction::new(0.8),
            Restitution::new(0.1),
            ExternalImpulse::default(),
            PointValue { value: 5 },
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cylinder.glb#collection"))),
            AsyncSceneCollider::default(),
            Transform::from_translation(random_location()),
        ));
    }
}
fn handle_next_level(
    mut messages: MessageReader<NextLevel>,
    configuration: Res<Configuration>,
    mut old_balls: Query<Entity, With<BouncyBall>>,
    mut old_toys: Query<Entity, With<ToyType>>,
    mut old_barriers: Query<Entity, With<Barrier>>,
    mut scoreboard: ResMut<ScoreBoard>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
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
//            commands.write_message(HelpMessage { help_type: HelpType::Score, text: "No score yet".to_string() });
        } else if scoreboard.score == 0 {
            commands.write_message(HelpMessage { help_type: HelpType::Score, text: format!("No score for level {}", scoreboard.level) });
        }
        commands.write_message(HelpMessage { help_type: HelpType::Next, text: "Press Enter to drop a ball".to_string() });
        //        println!("Level: {}", scoreboard.level);
        commands.write_message(ActivateGameMessage {});
        let toys = configuration.get_toys(scoreboard.level);
//        println!("Level {}, Toys {:?}", scoreboard.level, toys);
        create_blocks(toys.blocks, &mut commands, &mut meshes, &mut materials);
        create_barriers(toys.barriers, &mut commands, &mut meshes, &mut materials);
        create_disks(toys.disks, &mut commands, &mut meshes, &mut materials);
        create_cones(toys.cones, &mut commands, &mut meshes, &mut materials);
        create_dips(toys.dips, &mut commands, &mut meshes, &mut materials, &asset_server);
        create_blacks(toys.blacks, &mut commands, &mut meshes, &mut materials);
        create_bumpys(toys.bumpys, &mut commands, &asset_server);
        create_targets(toys.targets, &mut commands, &mut meshes, &mut materials);
        create_ghosts(toys.ghosts, &mut commands, &mut meshes, &mut materials);
        create_lifesavers(toys.lifesavers, &mut commands, &asset_server);
        create_cylinders(toys.cylinders, &mut commands, &asset_server);
        println!("Done creating toys");
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
    mut commands: Commands,
) {
//    println!("Sending next level from start_next_Level");
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
            commands.write_message(HelpMessage { help_type: HelpType::Score, text: "You have no balls".to_string() });
        } else {
            commands.write_message(HelpMessage { help_type: HelpType::Next, text: "First, press G to start a game".to_string() });
        }
        return;
    }
    if scoreboard.level == 1 {
        commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Use arrow keys to move the ball around\nand push toys off the edge".to_string()});
    } else if scoreboard.level == 2 {
        commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Use space bar to bounce the ball".to_string()});
    } else if scoreboard.level == 3 {
        commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Hit the toys on the scoreboard, too".to_string()});
    } else if scoreboard.level == 5 {
        commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Hit the top of the black toy to turn it white".to_string()});
    } else {
        commands.write_message( HelpMessage{help_type: HelpType::Next, text: "Go for it".to_string()});
    }
    scoreboard.use_a_ball();
    // Make any live balls dead, usually only one
    for (mut bouncyball, mut point_value, material_handle) in query.iter_mut() {
        if bouncyball.live {
            bouncyball.live = false;
            point_value.value = 2;
            if let Some(mut material) = materials.get_mut(material_handle) {
                material.base_color = DEAD_BALL;
            }
        }
    }
    let mut rng = rand::rng();
    let x_pos: f32 = rng.random_range(-12.0..0.);
    let y_pos: f32 = rng.random_range(15.0..25.);
    let z_pos: f32 = rng.random_range(-9.0..9.0);
    // Spawn a Dynamic Bouncing Ball
    commands.spawn((
        BouncyBall{live: true},
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
    mut balls: Query<(&mut ExternalImpulse, &BouncyBall), With<BouncyBall>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    // Just interested in the live ball
    for (mut impulse, ball) in balls.iter_mut() {
        if ball.live  {
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
    }
}

fn update_scoreboard(
    mut scoreboard_query: Query<&mut TextMesh, With<Score>>,
    scoreboard: Res<ScoreBoard>,
) {
    for mut text in scoreboard_query.iter_mut() {
        text.text = format!("Game Level: {}\nLevel Score: {}\nTotal Score: {}\nToys Left: {}\nBalls Left: {}",
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
        Visibility::Hidden,
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
            scale: Vec3::splat(0.8),
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
        CameraController{},
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
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
