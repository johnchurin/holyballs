#![feature(trivial_bounds)]
use std::env;
// Holy Balls 3d game
// Copyright (C) 2026 John Churin
// Suppress console output
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use bevy::audio::Volume;
use bevy::input::common_conditions::{input_just_pressed, input_just_released};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::f32::consts::{FRAC_PI_2};
use std::time::Duration;
use bevy::light::NotShadowCaster;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowMode};
use bevy::input::mouse::MouseMotion;
use bevy_rapier3d::rapier::prelude::CollisionEventFlags;
use rand::RngExt;
use bevy_fontmesh::{FontMeshPlugin, JustifyText, TextAnchor, TextMesh, TextMeshStyle};
use wasm_bindgen::prelude::wasm_bindgen;
use std::sync::LazyLock;
use bevy::asset::AssetMetaCheck;
use crossfire::*;
use crossfire::mpmc::Array;
use clap::{Args, CommandFactory, Parser, Subcommand};

#[wasm_bindgen(module = "/site/js/export.js")]
extern "C" {
    fn game_ended();
    fn console_message(msg: &str);
}
const BUMP: f32 = 2.5;

const BACKGROUND_COLOR: Color = Color::srgb(0.7, 0.8, 0.7);
//    const DEAD_BALL: Color = Color::srgb(0.9, 0.0, 0.9);
const LIVE_BALL: Color = Color::srgb(1.0, 0.0, 0.0);
const CONE_COLOR: Color = Color::srgb(1.0, 0.0, 1.0);
const DISK_COLOR: Color = Color::srgb(0.0, 0.9, 0.5);
const BOX_COLOR: Color = Color::srgb(0.0, 0.0, 1.0);
const BOX_COLOR_TRANSPARENT: Color = Color::srgba(0.0, 0.0, 1.0, 0.2);
const LIGHT_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const BARRIER_COLOR: Color = Color::srgb(1.0, 0.64, 0.0);
const TARGET_COLOR: Color = Color::srgb(255.0 / 255.0, 105.0 / 255.0, 180.0 / 255.0);
const FLOOR_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);
const FENCE_COLOR: Color = Color::srgba(0.0, 0.9, 0.0, 0.4);
const BLACK_DISK_COLOR: Color = Color::srgb(0.0, 0.0, 0.0);
const WHITE_DISK_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const SCOREBOARD_COLOR: Color = Color::srgb(173.0/255.0, 216.0/255.0, 230.0/255.0);
const TEXT_COLOR: Color = Color::srgb(0.2, 0.2, 0.2);
//    const CYLINDER_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
const _CYLINDER_HALF_HEIGHT: f32 = 2.0;
const BALL_GROUP: Group = Group::GROUP_1;  // 0b0001
const TOY_GROUP: Group = Group::GROUP_2;   // 0b0010
const FENCE_GROUP: Group = Group::GROUP_3;  // 0b0100
const FIXED_GROUP: Group = Group::GROUP_4;  // 0b0100

const CONTINUOUS_PLAY: bool = true;

#[derive(Resource)]
struct ExternalChannel {
    tx: MAsyncTx<Array<Vec<String>>>,
    rx: MAsyncRx<Array<Vec<String>>>,
}

impl ExternalChannel {
    fn new() -> Self {
        let (tx, rx) = mpmc::bounded_async::<Vec<String>>(3);
        Self { tx, rx }
    }
    fn send(&self, message: Vec<String>) {
        self.tx.try_send(message).expect("No tx channel");
    }
}
static EXTERNAL_CHANNEL: LazyLock<ExternalChannel> = LazyLock::new(|| { ExternalChannel::new() });

#[wasm_bindgen]
pub fn execute(
    args: Vec<String>,
) {
    EXTERNAL_CHANNEL.send(args);
//    console_message("Rust: In start game");
}
pub fn main() {
    let args: Vec<String> = env::args().collect();
    // The first element is always the path to the executable
    if args.len() > 0 {
        execute(args);
    }

    App::new()
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    canvas: Some("#game-canvas".into()),
                    title: "Holy Balls".into(),
                    fit_canvas_to_parent: true,
                    prevent_default_event_handling: false,
                    ..default()
                }),
            ..default()
            })
            .set(AssetPlugin{
                meta_check: AssetMetaCheck::Never,
                ..default()
            })
        )
        // Initialize the Rapier physics engine
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(FontMeshPlugin::<StandardMaterial>::default())
        //        .add_systems(Startup, setup_window)
        .add_systems(Startup, setup_configuration)
        .add_systems(Startup, setup_game_board)
        .insert_resource(ClearColor(BACKGROUND_COLOR))
        .insert_resource(Scoreboard::new(EXTERNAL_CHANNEL.rx.clone()))
        .insert_resource(CountdownBoard::new())
        .insert_resource(GlobalVolume::new(Volume::Linear(0.25)))
        .insert_resource(ScoringHelpTimer::new(4.0))
        .insert_resource(Configuration::new())
        .insert_resource(GameLevelResource::new())
        .insert_resource(ExitDelay{ seconds: None})
        .add_systems(Update, (
            toggle_overhead_camera.run_if(input_just_pressed(KeyCode::KeyO)),
            drop_a_ball.run_if(input_just_pressed(KeyCode::Enter)),
            drop_a_ball.run_if(input_just_pressed(KeyCode::NumpadEnter)),
            mouse_down.run_if(input_just_pressed(MouseButton::Left)),
            mouse_up.run_if(input_just_released(MouseButton::Left)),
            impulse.run_if(input_just_pressed(KeyCode::Space)),
            impulse.run_if(input_just_pressed(KeyCode::Numpad5)),
            impulse.run_if(input_just_pressed(KeyCode::ArrowLeft)),
            impulse.run_if(input_just_pressed(KeyCode::Numpad4)),
            impulse.run_if(input_just_pressed(KeyCode::ArrowRight)),
            impulse.run_if(input_just_pressed(KeyCode::Numpad6)),
            impulse.run_if(input_just_pressed(KeyCode::ArrowDown)),
        ))
        .add_systems(Update, (
            impulse.run_if(input_just_pressed(KeyCode::Numpad2)),
            impulse.run_if(input_just_pressed(KeyCode::ArrowUp)),
            impulse.run_if(input_just_pressed(KeyCode::Numpad8)),
            start_new_game.run_if(input_just_pressed(KeyCode::KeyG)),
            start_next_level.run_if(input_just_pressed(KeyCode::KeyN)),
            restart_same_level.run_if(input_just_pressed(KeyCode::KeyR)),
            update_scoreboard.run_if(resource_changed::<Scoreboard>),
            update_countdown_face.run_if(resource_changed::<CountdownBoard>),
            // mouse_look_system.run_if(|mouse: Res<ButtonInput<MouseButton>>| mouse.pressed(MouseButton::Left)),
        ))
        .add_systems(Update, update_countdown)
        .add_systems(Update, (
            clear_scoring_text,
            check_external_channel,
        ))
        .add_systems(Update, (
            handle_exit.run_if(input_just_pressed(KeyCode::KeyX)),
            handle_mouse_move,
            handle_point_value_message,
            handle_activate_game,
            handle_impulse_message,
        ))
        .add_systems(Update, (
            handle_new_level,
            handle_sensor_events,
            handle_help_message,
            handle_sound,
            handle_asset_color_propagation,
            score_fallen_entities,
            handle_ball_message,
        ))
        .add_message::<BallMessage>()
        .add_message::<PointValueMessage>()
        .add_message::<PropagateAssetColorMessage>()
        .add_message::<ActivateGameMessage>()
        .add_message::<ImpulseMessage>()
        .add_message::<HelpMessage>()
        .add_message::<PlayLevel>()
        .add_message::<SoundMessage>()
        .run();
}
#[derive(Resource)]
struct ExitDelay {
    seconds: Option<f32>,
}
#[derive(Resource)]
struct ScoringHelpTimer {
    entity: Option<Entity>,
    start: f32,
    duration: f32,
    active: bool,
}
#[derive(Resource)]
struct GameLevelResource {
    game_level: Option<GameLevel>,
}
impl GameLevelResource {
    fn new() -> Self {
        Self {
            game_level: None,
        }
    }
    fn set_game_level(&mut self, game_level: &GameLevel) {
        self.game_level = Some(game_level.clone());
    }
    fn clear_game_level(&mut self) {
        self.game_level = None;
    }
}
#[derive(Default, Clone, Debug)]
struct GameLevel {
    seconds: Option<Duration>,
    balls: i32,
    barriers: i32,
    blocks: i32,
    disks: i32,
    cones: i32,
    blacks: i32,
    dips: i32,
    bumpys: i32,
    targets: i32,
    spikeys: i32,
    ghosts: i32,
    lifesavers: i32,
    cylinders: i32,
    fences: i32,
    wind: Option<Vec3>,
    help: String,
}
#[derive(Resource)]
struct Configuration {
    levels: Vec<GameLevel>,
}
impl Configuration {
    fn new() -> Self {
        Self { levels: Vec::new() }
    }

    fn add(&mut self, level: GameLevel) -> &mut Self {
        self.levels.push(level);
        self
    }

    fn get_game_level(&self, level: i32) -> &GameLevel {
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
    fn start(&mut self, entity: Entity, start: f32) {
        self.entity = Some(entity);
        self.active = true;
        self.start = start;
    }
}

enum SoundType {
    Win,
    Lose,
    Bonus,
    NewLevel,
    GameOver,
    FinishLevel,
}
#[derive(Message)]
struct ImpulseMessage {
    entity: Entity,
    force: Vec3,
}

#[derive(Message)]
struct ActivateGameMessage {}
#[derive(Message, Debug)]
struct PropagateAssetColorMessage {
    entity: Entity,
    color: Color,
}
#[derive(Message, Debug)]
struct PointValueMessage {
    entity: Entity,
    value: i32,
}
#[derive(Message)]
struct SoundMessage {
    sound_type: SoundType,
}
#[derive(Message)]
struct BallMessage {
}
#[derive(Message)]
struct PlayLevel {}
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
struct Score {}
#[derive(Component)]
struct HelpWall {}
#[derive(Component)]
struct ClockBoard {}
#[derive(Component)]
struct ClockBoardFace {}
#[derive(Component)]
struct ScoringWall {}

#[derive(Component, Clone, Copy, Debug)]
struct BouncyBall {
    live: bool,
}
#[derive(Component)]
struct Barrier {}

#[derive(Component, Debug)]
struct SensorChild {
    next_color: Color,
}

#[derive(Component)]
struct PointValue {
    value: i32,
}

// enum FenceType {
//     Back,
//     Left,
//     Right,
//     Front
// }
#[derive(Component)]
struct Fence {
    //        fence_type: FenceType,
}

#[derive(Component)]
struct ToyType {
    dynamic: bool,
}
#[derive(Resource)]
struct CountdownBoard {
    goal: Option<Duration>,
    countdown: Option<Duration>,
}
impl CountdownBoard {
    fn new() -> Self {
        Self { goal: None, countdown: None }
    }
    fn start(&mut self, goal: Option<Duration>) {
        self.goal = goal;
        self.countdown = goal;
    }
    // Return true if the countdown is over
    fn reduce_countdown(&mut self, tick: Duration) {
        if self.countdown != None {
            self.countdown = self.countdown.unwrap().checked_sub(tick);
        }
    }
    fn is_running(&self) -> bool {
        self.countdown != None
    }
}

#[derive(Resource)]
struct Scoreboard {
    running: bool,
    score: i32,
    level: i32,
    total: i32,
    toys: i32,
    balls: i32,
    sound: bool,
    starting_level: i32,
    rx: MAsyncRx<Array<Vec<String>>>,
}

impl Scoreboard {
    fn new(rx: MAsyncRx<Array<Vec<String>>>) -> Self {
        Self { running: false, score: 0, level: 0, total: 0, toys: 0, balls: 0, sound: false, starting_level: 0, rx }
    }
    fn get_message(&mut self) -> Option<Vec<String>> {
        let r = self.rx.try_recv();
        if r.is_ok() {
            Some(r.unwrap())
        } else {
            None
        }
    }
    fn hit(&mut self, incr: i32) {
        self.score += incr;
        self.total += incr;
    }
    fn set_starting_level(&mut self, level: i32) {
        self.level = level-1;
        self.starting_level = level;
    }

    fn set_sound(&mut self, sound: bool) {
        self.sound = sound;
    }

    fn use_a_ball(&mut self) {
        self.balls -= 1;
    }
    fn set_balls_count(&mut self, count: i32) {
        self.balls = count;
    }

    fn stop(&mut self) {
        println!("stopping game");
        self.running = false;
    }
    fn start(&mut self) {
        println!("starting level {}", self.starting_level);
        self.running = true;
    }
    fn next_level(&mut self) {
        self.score = 0;
        self.level += 1;
        //        self.balls = 3;
    }
    fn _same_level(&mut self) {
        self.score = 0;
        //        self.level += 1;
        //        self.balls = 3;
    }
    fn reset(&mut self) {
        println!("reset scoreboard");
        self.running = false;
        self.score = 0;
        self.level = self.starting_level-1;
        self.total = 0;
        self.balls = 0;
    }
}
#[derive(Component)]
struct CameraController {}

fn setup_window(
    // mut win_query: Query<&mut Window, With<PrimaryWindow>>,
    mut co_query: Query<&mut CursorOptions>,
) {
//    let mut win = win_query.single_mut().unwrap();
    //        window.set_maximized(true);
    //        window.mode = WindowMode::Windowed;
//    win.mode = WindowMode::BorderlessFullscreen { 0: MonitorSelection::Current };
//     win.title = "Holy Balls".into();
//     win.canvas = Some("#game-canvas".into());
    let mut co = co_query.single_mut().unwrap();
    co.visible = false;
    co.grab_mode = CursorGrabMode::Locked;
}
#[derive(Parser, Debug)]
#[command()]
struct ExecuteArgs {
    #[command(subcommand)]
    command: ExecuteCommands,
}
#[derive(Subcommand, Debug)]
enum ExecuteCommands {
    /// Start the game
    Start(StartArgs),
    /// Exiy the game
    Exit {},
}
#[derive(Args, Debug)]
struct StartArgs {
    #[arg(short, long, default_value_t = false)]
    sound: bool,

    #[arg(short, long, default_value_t = 1)]
    level: i32,
}

fn check_external_channel(
    time: Res<Time>,
    mut commands: Commands,
    mut exit_delay: ResMut<ExitDelay>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    while let Some(message) = scoreboard.get_message() {
//        println!("Received: {}", message);
//        let message = vec!("xxx", "help", "start");
//        let text = message.join(" ");
//        console_message(format!("Rust: In check_external_channel: {}", text).as_str());
        let cli = ExecuteArgs::try_parse_from(message);
        if cli.is_ok() {
            let mut cmd = ExecuteArgs::command();
            // let help = cmd.render_long_help();
            // console_message(help.to_string().as_str());
            match cli.unwrap().command {
                ExecuteCommands::Start(args) => {
//                    console_message("Match on start");
                    scoreboard.reset();
                    scoreboard.set_starting_level(args.level);
                    scoreboard.set_sound(args.sound);
                    scoreboard.next_level();
                    //    println!("Sending next level from start_new_game");
                    commands.write_message(PlayLevel {});
                }
                ExecuteCommands::Exit {} => {
//                    console_message("Match on exit");
                    //                exit_delay.seconds = Some(0.0);
                    #[cfg(target_arch = "wasm32")]
                    game_ended();
                }
            }
        } else {
            console_message(format!("Execute: {}", cli.err().unwrap()).as_str());
        }
    }
    // Delayed exit
    if exit_delay.seconds.is_some() {
        let delay = exit_delay.seconds.unwrap();
        if delay <= 0.0 {
            exit_delay.seconds = None;
            #[cfg(target_arch = "wasm32")]
            game_ended();
            #[cfg(not(target_arch = "wasm32"))]
            commands.write_message(AppExit::Success);
        } else {
            exit_delay.seconds = Some(delay-time.delta_secs());
        }
    }
}
fn handle_exit(
    mut commands: Commands,
)
{
    execute(vec!("internal".to_string(),"exit".to_string(), ));
}

fn setup_configuration(
    mut configuration: ResMut<Configuration>,
) {
    configuration.add(GameLevel {
        balls: 5,
        blocks: 1,
        fences: 4,
        help: "If you lose a ball over the edge, another one will fall automatically.".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        balls: 5,
        fences: 3,
        blocks: 2,
        disks: 2,
        help: "More toys to push off the edge.\n\
        Caution: No fence on the front edge.".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        seconds: Some(Duration::from_mins(2)),
        balls: 3,
        fences: 1,
        blocks: 1,
        disks: 1,
        help: "Fewer balls available now\n\
        and, the levels are timed.".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        seconds: Some(Duration::from_mins(2)),
        balls: 3,
        barriers: 1,
        blocks: 2,
        help: "Use the space bar to bounce the ball over the barrier".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        seconds: Some(Duration::from_secs(30)),
        balls: 3,
        barriers: 1,
        blocks: 2,
        help: "Adding some time pressure, you only have 30 seconds!".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        seconds: Some(Duration::from_mins(3)),
        balls: 3,
        barriers: 2,
        blacks: 1,
        help: "Hit the top of the black disk to turn it white and\n\
        get bonus points when you push it off the edge".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        seconds: Some(Duration::from_mins(2)),
        balls: 3,
        barriers: 2,
        blocks: 2,
        wind: Some(Vec3::new(0.3, 0.0, 0.0)),
        help: "This time with a light breeze coming out of the west".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        seconds: Some(Duration::from_mins(2)),
        balls: 3,
        barriers: 3,
        blocks: 2,
        help: "Is there a toy hiding behind the barrier? Toggle the O key to check.".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        seconds: Some(Duration::from_mins(2)),
        balls: 3,
        barriers: 4,
        disks: 2,
        help: "More hiding places".to_string(),
        ..GameLevel::default()
    });
    configuration.add(GameLevel {
        seconds: Some(Duration::from_mins(4)),
        balls: 3,
        barriers: 2,
        targets: 1,
        help: "Balls to the walls: Hit the disk on the scoreboard.\n\
        If it lands on the table, you have to push it off the edge\n\
        to get points.".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        seconds: Some(Duration::from_mins(3)),
        balls: 3,
        barriers: 2,
        ghosts: 4,
        help: "Some ghost blocks. 3 minutes to clear the toys.".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        seconds: Some(Duration::from_mins(2)),
        balls: 3,
        barriers: 2,
        spikeys: 2,
        help: "Spikey balls are a bit of a challenge to get over the edge.".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        seconds: Some(Duration::from_mins(3)),
        balls: 3,
        barriers: 2,
        lifesavers: 2,
        help: "Put the ball in the lifesaver to earn bonus points.".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        seconds: Some(Duration::from_mins(3)),
        balls: 3,
        barriers: 2,
        dips: 2,
        help: "Put the ball in the dip to turn the piece white to\n\
        get bonus points when you push it off the edge".to_string(),
        ..GameLevel::default()
    });

    configuration.add(GameLevel {
        seconds: Some(Duration::from_secs(45)),
        balls: 3,
        barriers: 2,
        blocks: 6,
        ghosts: 2,
        cylinders: 3,
        lifesavers: 2,
        help: "More toys to push off the edge. Not much time to do it.".to_string(),
        ..GameLevel::default()
    });
    configuration.add(GameLevel {
        seconds: Some(Duration::from_mins(5)),
        balls: 3,
        barriers: 2,
        blocks: 6,
        ghosts: 4,
        blacks: 3,
        dips: 3,
        targets: 3,
        cylinders: 4,
        spikeys: 1,
        lifesavers: 2,
        help: "Lots of toys. Good luck.".to_string(),
        ..GameLevel::default()
    });
}
fn clear_scoring_text(
    time: Res<Time>,
    mut timer: ResMut<ScoringHelpTimer>,
    mut query: Query<&mut Visibility>,
) {
    if timer.active {
        if timer.entity.is_some() && time.elapsed_secs() > timer.start + timer.duration {
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
fn update_countdown(
    mut commands: Commands,
    time: Res<Time>,
    mut scoreboard: ResMut<Scoreboard>,
    mut countdown_board: ResMut<CountdownBoard>,
    mut exit_delay: ResMut<ExitDelay>,
) {
    if scoreboard.running && countdown_board.is_running() {
        countdown_board.reduce_countdown(time.delta());
        if !countdown_board.is_running() {
            scoreboard.stop();
            commands.write_message(HelpMessage { help_type: HelpType::Next,
                text: "Game Over.".to_string() });
            commands.write_message(HelpMessage { help_type: HelpType::Score,
                text: "You are out of time.".to_string() });
            // Wha Wha Wha them out
            commands.write_message(SoundMessage { sound_type: SoundType::GameOver });
            // Game over. Leave, but slowly
            exit_delay.seconds = Some(5.0);
        }
    }
}
// Restore the cursor when left button released
fn mouse_up(
    mut co_query: Query<&mut CursorOptions>,
) {
    let mut co = co_query.single_mut().unwrap();
    co.visible = true;
    co.grab_mode = CursorGrabMode::None;
}
// Hide and capture the cursor during mouse movement
fn mouse_down(
    mut co_query: Query<&mut CursorOptions>,
) {
    let mut co = co_query.single_mut().unwrap();
    co.visible = false;
    co.grab_mode = CursorGrabMode::Locked;
}
fn handle_mouse_move(
    mut messages: ResMut<Messages<MouseMotion>>,
    // mut commands: Commands,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut balls: Query<(&mut ExternalImpulse, &BouncyBall), With<BouncyBall>>,
    //    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    for event in messages.drain() {
       if mouse_buttons.pressed(MouseButton::Left) {
            for (mut impulse, _ball) in balls.iter_mut() {
                //                info!("Mouse moved: x = {}, y = {}", event.delta.x, event.delta.y);
                impulse.impulse = Vec3::new(event.delta.x * 0.1, 0.0, event.delta.y * 0.1);
            }
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
                        if !bouncy_ball.live { continue } // live balls only
                        // Sort out which of the pair is the sensor (the other one is the ball)
                        let sensor = if ball_entity == entity2 { entity1 } else if ball_entity == entity1 { entity2 } else { continue };
                        // The ball collides with the SensorChild, not its parent toy
                        for (sensor_entity, parent, mut child_point_value, sensor_child) in sensor_query.iter_mut() {
                            // Now get the toy so we can add the points in
                            if sensor == sensor_entity {
                                if let Ok(toy_entity) = toy_query.get_mut(parent.0) {
//                                    println!("parent.0: {:?}, toy: {:?}", parent.0.entity(), toy_entity);
                                    if child_point_value.value != 0 {
                                        commands.write_message(PointValueMessage { entity: toy_entity, value: child_point_value.value });
                                        commands.write_message(HelpMessage { help_type: HelpType::Score, text: "You get bonus points for that task".to_string() });
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
                        if !bouncy_ball.live { continue }
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
    ball_query: Query<(Entity, &BouncyBall, &mut Transform, &PointValue), (With<BouncyBall>, Without<ToyType>)>,
    toy_query: Query<(Entity, &Transform, &PointValue), (With<ToyType>, Without<BouncyBall>)>,
    mut scoreboard: ResMut<Scoreboard>,
    mut exit_delay: ResMut<ExitDelay>,
) {
    scoreboard.toys = 0;
    // score and cleanup old toys and balls that are out of range
    for (entity, transform, point_value) in toy_query.iter() {
        //        scoreboard.remaining += 1;
        if transform.translation.y < -15.0 {
            println!("Toy despawned {} points", point_value.value);
            if scoreboard.running {
                scoreboard.hit(point_value.value);
                if point_value.value > 0 {
                    commands.write_message(HelpMessage { help_type: HelpType::Score, text: format!("Toy overboard. You scored {} points!", point_value.value) });
                } else {
                    commands.write_message(HelpMessage { help_type: HelpType::Score, text: format!("Toy overboard. You lost {} points", -point_value.value) });
                }
                if point_value.value != 0 && scoreboard.running {
                    commands.write_message(
                        SoundMessage {
                            sound_type: if point_value.value < 0
                            { SoundType::Lose } else { SoundType::Win }
                        });
                }
            }
            if let Ok(mut entity_cmds) = commands.get_entity(entity) {
                entity_cmds.despawn();
            }
        } else {
            // Put the toy count back as we found it.
            scoreboard.toys += 1;
        }
    }
    // We're out of toys, so it's time for a new level.
    if scoreboard.running && scoreboard.toys == 0 {
        // The level is no longer running
        scoreboard.stop();
//        println!("{} toys found", scoreboard.toys);
        scoreboard.hit(100);
        commands.write_message(HelpMessage {
            help_type: HelpType::Score,
            text: "100 bonus points for clearing this level".to_string()
        });
        commands.write_message(SoundMessage { sound_type: SoundType::FinishLevel });
        if CONTINUOUS_PLAY {
            start_next_level(scoreboard, commands);
        } else {
            let text = format!("Press N to start level {}", scoreboard.level + 1);
            commands.write_message(HelpMessage { help_type: HelpType::Next, text });
        }
        return;
    }
    // Look for any fallen balls
    for (entity, ball, transform, point_value) in ball_query.iter() {
        if transform.translation.y < -15.0 {
            if scoreboard.running {
                scoreboard.hit(point_value.value);
                if point_value.value != 0 {
                    if point_value.value > 0 {
                        commands.write_message(HelpMessage { help_type: HelpType::Score, text: format!("Ball overboard. You scored {} points", point_value.value) });
                    } else {
                        commands.write_message(HelpMessage { help_type: HelpType::Score, text: format!("Ball overboard. You lost {} points", -point_value.value) });
                    }
                    // If this is our last, live ball, then game over.
                    if scoreboard.balls == 0 {
                        if ball.live {
                            commands.write_message(HelpMessage { help_type: HelpType::Next, text: "Game Over.".to_string() });
                            // Wha Wha Wha them out
                            commands.write_message(SoundMessage { sound_type: SoundType::GameOver });
                            exit_delay.seconds = Some(5.0);
                        }
                    } else {
                        commands.write_message(
                            SoundMessage {
                                sound_type: if point_value.value < 0
                                { SoundType::Lose } else { SoundType::Win }
                            });
                        if CONTINUOUS_PLAY {
                            commands.write_message(BallMessage {});
                        } else {
                            commands.write_message(HelpMessage {
                                help_type: HelpType::Score,
                                text: "Press Enter for another ball".to_string()
                            });
                        }
                    }
                }
            }
            if let Ok(mut entity_cmds) = commands.get_entity(entity) {
                entity_cmds.despawn();
            }
        }
    }
}

fn handle_help_message(
    mut messages: MessageReader<HelpMessage>,
    mut help_query: Query<&mut TextMesh, (With<HelpWall>, Without<ScoringWall>)>,
    mut score_query: Query<(Entity, &mut TextMesh, &mut Visibility), (With<ScoringWall>, Without<HelpWall>)>,
    mut scoring_help_timer: ResMut<ScoringHelpTimer>,
    time: Res<Time>,
) {
    for message in messages.read() {
        match message.help_type {
            HelpType::Score => {
                for (entity, mut text_mesh, mut visibility) in score_query.iter_mut() {
                    *visibility = Visibility::Visible {};
                    if scoring_help_timer.active && !text_mesh.text.ends_with(message.text.as_str()) {
                        text_mesh.text += "\nand ";
                        text_mesh.text += message.text.as_str();
                    } else {
                        text_mesh.text = message.text.clone();
                    }
                    scoring_help_timer.start(entity, time.elapsed_secs());
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
fn handle_point_value_message(
    mut messages: MessageReader<PointValueMessage>,
    mut query: Query<&mut PointValue, With<ToyType>>,
) {
    for message in messages.read() {
        if let Ok(mut point_value) = query.get_mut(message.entity) {
            point_value.value += message.value;
        }
    }
}
fn handle_impulse_message(
    mut messages: MessageReader<ImpulseMessage>,
    mut query: Query<(Entity, &mut ExternalImpulse), With<ToyType>>,
    mut commands: Commands,
) {
    for message in messages.read() {
        if commands.get_entity(message.entity).is_ok() {
            for (entity, mut external_impulse) in query.iter_mut() {
                if entity == message.entity {
                    external_impulse.impulse = message.force;
                }
            }
        }
    }
}

fn handle_sound(
    mut messages: MessageReader<SoundMessage>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    scoreboard: Res<Scoreboard>,
) {
    for event in messages.read() {
        console_message("Sound playing");
        if scoreboard.sound {
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
                SoundType::GameOver => {
                    commands.spawn((
                        AudioPlayer::new(asset_server.load("audio/wha-wha.ogg")),
                        PlaybackSettings::ONCE,
                    ));
                }
                SoundType::NewLevel => {
                    commands.spawn((
                        AudioPlayer::new(asset_server.load("audio/intro.ogg")),
                        PlaybackSettings::ONCE,
                    ));
                }
                SoundType::FinishLevel => {
                    commands.spawn((
                        AudioPlayer::new(asset_server.load("audio/fanfare.ogg")),
                        PlaybackSettings::ONCE,
                    ));
                }
            }
        }
    }
}
fn random_location() -> Vec3 {
    let mut rng = rand::rng();
    Vec3::new(rng.random_range(-10..10) as f32,
              10.0 + rng.random_range(0.0..10.0),
              rng.random_range(-9..9) as f32)
}

fn handle_activate_game(
    mut messages: MessageReader<ActivateGameMessage>,
    //    mut commands: Commands,
    mut scoreboard: ResMut<Scoreboard>,
) {
    for _event in messages.read() {
        println!("Next level key press");
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
                if let Ok(mut entity_cmds) = commands.get_entity(message.entity) {
                    entity_cmds.insert(MeshMaterial3d(materials.add(unique_material)));
                }

//                commands.entity(message.entity).insert(MeshMaterial3d(materials.add(unique_material)));
            }
        }
        // Then recurse to the children, if any
        if let Ok(children) = children_query.get(message.entity) {
            for child in children.iter() {
                //                println!("Child Entity: {}", child);
                // And for good measure, continue down to other descendents
                commands.write_message(PropagateAssetColorMessage { entity: child, color: message.color });
            }
        }
    }
}
fn create_countdown_board(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &Res<AssetServer>,
) {
    let font = asset_server.load("fonts/digital_clock.ttf");
    commands.spawn((
        ClockBoard {},
        CollisionGroups::new(FIXED_GROUP, BALL_GROUP | TOY_GROUP),
        RigidBody::Fixed,
        Friction::new(0.5),
        Restitution::new(0.1),
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.5, 3.0, 8.0)))),
        MeshMaterial3d(materials.add(SCOREBOARD_COLOR)),
        Collider::cuboid(0.25, 1.5, 4.0),
        Transform::from_xyz(14.5, 5.0, 0.0),
    ));
    // .with_children(|parent| {
    //     parent.spawn((
    commands.spawn((
        ClockBoardFace {},
        Visibility::Hidden,
        TextMesh {
            text: "00:00".to_string(),
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
            base_color: TEXT_COLOR,
            //            metallic: 0.8,
            perceptual_roughness: 0.3,
            reflectance: 0.8,
            double_sided: false,
            cull_mode: None,
            ..default()
        })),
        Transform {
            translation: Vec3::new(14.25, 5.0, 0.0),
            rotation: Quat::from_axis_angle(Vec3::Y, -FRAC_PI_2),   // 90 degrees
            scale: Vec3::splat(3.0),
        }
    ));
}
fn create_barriers(
    game_level: &GameLevel,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    //    asset_server: AssetServer,
) {
    let n = game_level.barriers;
    let hy = if n > 2 { 1.0 } else { 0.25 };
    if n > 0 {
        // Barrier Left
        commands.spawn((
            Barrier {},
            RigidBody::Fixed,
            Friction::new(0.0),
            Restitution::new(0.1),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(17.0, hy*2.0,2.0)))),
            MeshMaterial3d(materials.add(BARRIER_COLOR)),
            Collider::cuboid(8.5, hy, 1.0),
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
    if n > 3 {
        // Barrier wall
        commands.spawn((
            Barrier {},
            RigidBody::Fixed,
            Friction::new(0.0),
            Restitution::new(0.1),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(1.0, 2.5, 20.0)))),
            MeshMaterial3d(materials.add(BARRIER_COLOR)),
            Collider::cuboid(0.5, 1.25, 10.0),
            Transform::from_xyz(5.0, 1.25, 0.0),
        ));
    }
}
fn create_fences(
    game_level: &GameLevel,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let n = game_level.fences;
    if n == 1 || n == 3 || n == 4 {
        commands.spawn((
            CollisionGroups::new(FENCE_GROUP, BALL_GROUP),
            NotShadowCaster,
            Fence { },
            RigidBody::Fixed,
            Friction::new(0.5),
            Restitution::new(0.1),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(25.0 - 0.25, 1.0, 0.25 / 2.0)))),
            MeshMaterial3d(materials.add(FENCE_COLOR)),
            Collider::cuboid(12.5 - 0.25, 0.5, 0.125 / 2.0),
            Transform::from_xyz(0.0, 0.5, -10.0),
        ));
    }
    if n == 2 || n == 3 || n == 4 {
        commands.spawn((
            CollisionGroups::new(FENCE_GROUP, BALL_GROUP),
            NotShadowCaster,
            Fence {  },
            RigidBody::Fixed,
            Friction::new(0.5),
            Restitution::new(0.1),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.25 / 2.0, 1.0, 20.0 - 0.25)))),
            MeshMaterial3d(materials.add(FENCE_COLOR)),
            Collider::cuboid(0.125 / 2.0, 0.5, 10.0 - 0.125),
            Transform::from_xyz(12.5 - 0.125, 0.5, 0.0),
        ));
    }
    if n == 4 {
        commands.spawn((
            CollisionGroups::new(FENCE_GROUP, BALL_GROUP),
            NotShadowCaster,
            Fence {  },
            RigidBody::Fixed,
            Friction::new(0.5),
            Restitution::new(0.1),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(25.0 - 0.25, 1.0, 0.25 / 2.0)))),
            MeshMaterial3d(materials.add(FENCE_COLOR)),
            Collider::cuboid(12.5 - 0.25, 0.5, 0.125 / 2.0),
            Transform::from_xyz(0.0, 0.5, 10.0),
        ));
    }
    if n == 2 || n == 3 || n == 4 {
        commands.spawn((
            CollisionGroups::new(FENCE_GROUP, BALL_GROUP),
            NotShadowCaster,
            Fence {  },
            RigidBody::Fixed,
            Friction::new(0.5),
            Restitution::new(0.1),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.25 / 2.0, 1.0, 20.0 - 0.25)))),
            MeshMaterial3d(materials.add(FENCE_COLOR)),
            Collider::cuboid(0.125 / 2.0, 0.5, 10.0 - 0.25),
            Transform::from_xyz(-12.5 + 0.125, 0.5, 0.0),
        ));
    }
}
fn make_external_force(game_level: &GameLevel) -> ExternalForce {
    if game_level.wind.is_some() {
        ExternalForce {
            force: game_level.wind.unwrap(),
            torque: Vec3::new(0.0, 0.0, 0.0),
        }
    } else {
        ExternalForce::default()
    }
}
fn create_ball(
    game_level: &GameLevel,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let external_force = make_external_force(game_level);
    // Spawn a Dynamic Bouncing Ball
    commands.spawn((
        CollisionGroups::new(BALL_GROUP, TOY_GROUP | FENCE_GROUP | FIXED_GROUP),
        BouncyBall { live: true },
        RigidBody::Dynamic,
        PointValue { value: -10 },
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
        external_force,
        Transform::from_translation(random_location()),
        Velocity::linear(Vec3::new(2.0, 0.0, 0.0)),
        Mesh3d(meshes.add(Mesh::from(Sphere::new(0.5)))),
        MeshMaterial3d(materials.add(LIVE_BALL)),
    ));
}

fn create_blocks(
    game_level: &GameLevel,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    //    asset_server: AssetServer,
) {
    let external_force = make_external_force(game_level);
    // Boxes
    for _n in 0..game_level.blocks {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
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
            //            NotShadowCaster,
            ExternalImpulse::default(),
            external_force,
            PointValue { value: 15 },
            // Lower the Damping for a more advanced game
            // Damping {
            //     linear_damping: 0.2,
            //     angular_damping: 0.2,
            // },
            Collider::cuboid(0.5, 0.5, 0.5),
            Transform::from_translation(random_location()),
        ));
    };
}
fn create_disks(
    game_level: &GameLevel,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let external_force = make_external_force(game_level);
    for _n in 0..game_level.disks {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
            RigidBody::Dynamic,
            ToyType { dynamic: true },
            Friction::new(0.2),
            Restitution::new(0.1),
            Mesh3d(meshes.add(Mesh::from(Cylinder::new(0.75, 0.6)))),
            MeshMaterial3d(materials.add(DISK_COLOR)),
            ExternalImpulse::default(),
            external_force,
            PointValue { value: 15 },
            // Lower the Damping for a more advanced game
            // Damping {
            //     linear_damping: 0.2,
            //     angular_damping: 0.2,
            // },
            Collider::cylinder(0.3, 0.75),
            Transform::from_translation(random_location()),
        ));
    }
}

fn create_cones(
    game_level: &GameLevel,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let external_force = make_external_force(game_level);
    for _n in 0..game_level.cones {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
            ToyType { dynamic: true },
            RigidBody::Dynamic,
            Friction::new(0.1),
            Restitution::new(0.1),
            Mesh3d(meshes.add(Mesh::from(Cone::new(0.75, 2.0)))),
            MeshMaterial3d(materials.add(CONE_COLOR)),
            ExternalImpulse::default(),
            external_force,
            PointValue { value: 15 },
            // Lower the Damping for a more advanced game
            // Damping {
            //     linear_damping: 0.2,
            //     angular_damping: 0.2,
            // },
            Collider::cone(1.0, 0.75),
            Transform::from_translation(random_location()),
        ));
    }
}

fn create_blacks(
    game_level: &GameLevel,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    //    asset_server: AssetServer,
) {
    let external_force = make_external_force(game_level);
    for _n in 0..game_level.blacks {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
            RigidBody::Dynamic,
            ToyType { dynamic: true },
            Friction::new(0.2),
            ColliderMassProperties::Density(0.0),
            Restitution::new(0.1),
            Mesh3d(meshes.add(Mesh::from(Cylinder::new(0.75, 0.6)))),
            MeshMaterial3d(materials.add(BLACK_DISK_COLOR)),
            ExternalImpulse::default(),
            external_force,
            PointValue { value: -50 },
            // Lower the Damping for a more advanced game
            // Damping {
            //     linear_damping: 0.2,
            //     angular_damping: 0.2,
            // },
            Collider::cylinder(0.3, 0.75),
            Transform::from_translation(random_location()),
        )).with_children(|parent| {
            parent.spawn((
                SensorChild { next_color: WHITE_DISK_COLOR },
                Collider::ball(0.2),
                Sensor,
                PointValue { value: 100 },
                ActiveEvents::COLLISION_EVENTS,
                Transform::from_xyz(0.0, 0.6, 0.0),
            ));
            parent.spawn((
                SensorChild { next_color: WHITE_DISK_COLOR },
                Collider::ball(0.7),
                Sensor,
                PointValue { value: 100 },
                ActiveEvents::COLLISION_EVENTS,
                Transform::from_xyz(0.0, -0.6, 0.0),
            ));
        });
    }
}
fn create_dips(
    game_level: &GameLevel,
    commands: &mut Commands,
    _meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    asset_server: &Res<AssetServer>,
) {
    let external_force = make_external_force(game_level);
    for _n in 0..game_level.dips {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
            RigidBody::Dynamic,
            ToyType { dynamic: true },
            ExternalImpulse::default(),
            external_force,
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
            Transform::from_translation(random_location()).with_scale(Vec3::splat(0.5)).with_rotation(Quat::from_axis_angle(Vec3::Z, FRAC_PI_2 * 0.8)),
        )).with_children(|parent| {
            parent.spawn((
                SensorChild { next_color: WHITE_DISK_COLOR },
                Collider::ball(0.1),
                Sensor,
                PointValue { value: 25 },
                ActiveEvents::COLLISION_EVENTS,
                Transform::from_xyz(0.0, 0.2, 0.0),
            ));
        });
    }
}
fn create_bumpys(
    game_level: &GameLevel,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let external_force = make_external_force(game_level);
    for _n in 0..game_level.bumpys {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
            RigidBody::Dynamic,
            ToyType { dynamic: true },
            Friction::new(0.0),
            Restitution::new(0.1),
            ExternalImpulse::default(),
            external_force,
            PointValue { value: 5 },
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/bumpy.glb#collection"))),
            AsyncSceneCollider::default(),
            Transform::from_translation(random_location()),
        ));
    }
}
fn create_targets(
    game_level: &GameLevel,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let external_force = make_external_force(game_level);
    if game_level.targets > 0 {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
            ToyType { dynamic: false },
            RigidBody::Fixed,
            PointValue { value: 35 },
            ExternalImpulse::default(), // For when this becomes dynamic
            external_force,
            Friction::new(0.1),
            Restitution::new(0.1),
            Collider::cylinder(0.1, 0.5),
            Mesh3d(meshes.add(Mesh::from(Cylinder::new(0.50, 0.2)))),
            MeshMaterial3d(materials.add(TARGET_COLOR)),
            Transform::from_xyz(-14.0, 3.0, 5.0).with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
        ));
    }
    if game_level.targets > 1 {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
            ToyType { dynamic: false },
            RigidBody::Fixed,
            PointValue { value: 45 },
            ExternalImpulse::default(), // For when this becomes dynamic
            external_force,
            Friction::new(0.1),
            Restitution::new(0.1),
            Collider::cuboid(0.1, 1.0, 1.0),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.2, 1.5, 1.5)))),
            MeshMaterial3d(materials.add(TARGET_COLOR)),
            Transform::from_xyz(-14.0, 7.0, -5.0),
        ));
    }
    if game_level.targets > 2 {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
            ToyType { dynamic: false },
            RigidBody::Fixed,
            PointValue { value: 45 },
            ExternalImpulse::default(), // For when this becomes dynamic
            external_force,
            Friction::new(0.1),
            Restitution::new(0.1),
            Collider::cuboid(0.1, 1.0, 1.0),
            Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.2, 1.5, 1.5)))),
            MeshMaterial3d(materials.add(TARGET_COLOR)),
            Transform::from_xyz(-14.0, 7.0, 5.0),
        ));
    }
}

fn create_ghosts(
    game_level: &GameLevel,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let external_force = make_external_force(game_level);
    for _n in 0..game_level.ghosts {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
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
            external_force,
            PointValue { value: 20 },
            // Lower the Damping for a more advanced game
            // Damping {
            //     linear_damping: 0.2,
            //     angular_damping: 0.2,
            // },
            Collider::cuboid(0.5, 0.5, 0.5),
            Transform::from_translation(random_location()),
        ));
    };
}
fn create_lifesavers(
    game_level: &GameLevel,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let external_force = make_external_force(game_level);
    for _n in 0..game_level.lifesavers {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
            RigidBody::Dynamic,
            ToyType { dynamic: true },
            Friction::new(0.0),
            Restitution::new(0.1),
            ExternalImpulse::default(),
            external_force,
            PointValue { value: 10 },
            ColliderMassProperties::Density(0.50),
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/doughnut.glb#collection"))),
            AsyncSceneCollider::default(),
            Transform::from_translation(random_location()).with_scale(Vec3::splat(1.0)),
        )).with_children(|parent| {
            parent.spawn((
                SensorChild { next_color: WHITE_DISK_COLOR },
                Collider::ball(0.1),
                PointValue { value: 20 },
                Sensor,
                ActiveEvents::COLLISION_EVENTS,
                Transform::from_xyz(0.0, 0.2, 0.0),
            ));
        });
    }
}
fn create_spikeys(
    game_level: &GameLevel,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let external_force = make_external_force(game_level);
    for _n in 0..game_level.spikeys {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
            RigidBody::Dynamic,
            ToyType { dynamic: true },
            Friction::new(0.4),
            Restitution::new(0.1),
            ColliderMassProperties::Mass(0.25),
            ExternalImpulse::default(),
            external_force,
            PointValue { value: 25 },
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/spikey.glb#collection"))),
            AsyncSceneCollider::default(),
            Transform::from_translation(random_location()).with_scale(Vec3::splat(0.3)),
        ));
    }
}
fn create_cylinders(
    game_level: &GameLevel,
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
) {
    let external_force = make_external_force(game_level);
    for _n in 0..game_level.cylinders {
        commands.spawn((
            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP),
            RigidBody::Dynamic,
            ToyType { dynamic: true },
            Friction::new(0.8),
            Restitution::new(0.1),
            ExternalImpulse::default(),
            external_force,
            PointValue { value: 5 },
            WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cylinder.glb#collection"))),
            AsyncSceneCollider::default(),
            Transform::from_translation(random_location()),
        ));
    }
}
fn handle_new_level(
    mut messages: MessageReader<PlayLevel>,
    configuration: Res<Configuration>,
    mut old_balls: Query<Entity, With<BouncyBall>>,
    mut old_toys: Query<Entity, With<ToyType>>,
    mut old_barriers: Query<Entity, With<Barrier>>,
    mut scoreboard: ResMut<Scoreboard>,
    mut countdown_board: ResMut<CountdownBoard>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut game_level_res: ResMut<GameLevelResource>,
    asset_server: Res<AssetServer>,
    fence_query: Query<Entity, With<Fence>>,
    //    clock_query: Query<Entity, With<ClockBoard>>,
    mut clock_face_query: Query<&mut Visibility, With<ClockBoardFace>>,
) {
    for _event in messages.read() {
        if scoreboard.level < 1 {
            game_level_res.clear_game_level();
            commands.write_message(HelpMessage { help_type: HelpType::Score, text: "Press N to start the first level".to_string() });
            return;
        }
        // Remove old balls, toys, and barriers
        for entity in old_balls.iter_mut() {
            if let Ok(mut entity_cmds) = commands.get_entity(entity) {
                entity_cmds.despawn();
            }
        }
        for entity in old_toys.iter_mut() {
            if let Ok(mut entity_cmds) = commands.get_entity(entity) {
                entity_cmds.despawn();
            }
        }
        for entity in old_barriers.iter_mut() {
            if let Ok(mut entity_cmds) = commands.get_entity(entity) {
                entity_cmds.despawn();
            }
        }
        // Remove previous fences
        for entity in fence_query.iter() {
            if let Ok(mut entity_cmds) = commands.get_entity(entity) {
                entity_cmds.despawn();
            }
        }
        // Remove old clocks and clock faces, if any
        // for entity in clock_query.iter() {
        //     if let Ok(mut entity_cmds) = commands.get_entity(entity) {
        //         entity_cmds.despawn();
        //     }
        // }
        // for entity in clock_face_query.iter() {
        //     if let Ok(mut entity_cmds) = commands.get_entity(entity) {
        //         entity_cmds.despawn();
        //     }
        // }
        if !CONTINUOUS_PLAY {
            commands.write_message(HelpMessage { help_type: HelpType::Score, text: "Press Enter to drop a ball".to_string() });
        }
        //        println!("Level: {}", scoreboard.level);
        commands.write_message(ActivateGameMessage {});
        let game_level = configuration.get_game_level(scoreboard.level);
        game_level_res.set_game_level(game_level);
        //        println!("Level {}, Toys {:?}", scoreboard.level, toys);
        create_fences(game_level, &mut commands, &mut meshes, &mut materials);
        create_blocks(game_level, &mut commands, &mut meshes, &mut materials);
        create_barriers(game_level, &mut commands, &mut meshes, &mut materials);
        create_disks(game_level, &mut commands, &mut meshes, &mut materials);
        create_cones(game_level, &mut commands, &mut meshes, &mut materials);
        create_dips(game_level, &mut commands, &mut meshes, &mut materials, &asset_server);
        create_blacks(game_level, &mut commands, &mut meshes, &mut materials);
        create_bumpys(game_level, &mut commands, &asset_server);
        create_spikeys(game_level, &mut commands, &asset_server);
        create_targets(game_level, &mut commands, &mut meshes, &mut materials);
        create_ghosts(game_level, &mut commands, &mut meshes, &mut materials);
        create_lifesavers(game_level, &mut commands, &asset_server);
        create_cylinders(game_level, &mut commands, &asset_server);
        scoreboard.set_balls_count(game_level.balls);
        if game_level.seconds == None {
            for mut visibility in clock_face_query.iter_mut() {
                *visibility = Visibility::Hidden;
            }
        } else {
        // Start the clock
            countdown_board.start(game_level.seconds);
            // And make it visible
            for mut visibility in clock_face_query.iter_mut() {
                *visibility = Visibility::Visible;
            }
        }
        commands.write_message(HelpMessage { help_type: HelpType::Next, text: game_level.help.clone() });
        commands.write_message(SoundMessage { sound_type: SoundType::NewLevel });
        // Ready to drop a ball
        if CONTINUOUS_PLAY {
            commands.write_message(BallMessage{});
        }
    }
}
fn start_new_game(
    mut scoreboard: ResMut<Scoreboard>,
    mut commands: Commands,
) {
    scoreboard.reset();
    scoreboard.next_level();
    //    println!("Sending next level from start_new_game");
    commands.write_message(PlayLevel {});
    commands.write_message(HelpMessage { help_type: HelpType::Next, text: "Press the N key to start the first level".to_string() });
}

fn start_next_level(
    mut scoreboard: ResMut<Scoreboard>,
    mut commands: Commands,
) {
    scoreboard.next_level();
    //    println!("Sending next level from start_next_Level");
    commands.write_message(PlayLevel {});
}
fn restart_same_level(
    //        mut scoreboard: ResMut<Scoreboard>,
    mut commands: Commands,
) {
    //    scoreboard.same_level();
    commands.write_message(PlayLevel {});
}
fn drop_a_ball(
    mut commands: Commands,
) {
    commands.write_message(BallMessage{});
}
fn handle_ball_message(
    mut messages: MessageReader<BallMessage>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<Entity, With<BouncyBall>>,
    mut scoreboard: ResMut<Scoreboard>,
    game_level_res: Res<GameLevelResource>,
) {
    for _event in messages.read() {
        if scoreboard.balls == 0 {
            if scoreboard.running {
                commands.write_message(HelpMessage { help_type: HelpType::Score, text: "You have no balls".to_string() });
            }
            commands.write_message(HelpMessage { help_type: HelpType::Next, text: "Press G to start a game, or N to start the next level".to_string() });
            return;
        }
        // If another ball on the table, just quietly ignore the request.
        for _ball in query.iter() {
            return;
        }
        if game_level_res.game_level.is_some() {
            let game_level = game_level_res.game_level.as_ref().unwrap();
            // Update scoreboard
            scoreboard.use_a_ball();
            create_ball(game_level, &mut commands, &mut meshes, &mut materials);
        }
    }
}

fn impulse(
    mut balls: Query<(&mut ExternalImpulse, &BouncyBall), With<BouncyBall>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut commands: Commands,
) {
    // Just interested in the live ball
    if !CONTINUOUS_PLAY {
        let balls = balls.iter_mut();
        if balls.len() == 0 {
            commands.write_message(HelpMessage { help_type: HelpType::Score, text: "Press enter to get a fresh ball".to_string() });
            return;
        }
    }
    for (mut impulse, ball) in balls {
        if ball.live {
            // See which key was pressed
            for key in keyboard_input.get_just_pressed() {
                match key {
                    KeyCode::Space => {
                        impulse.impulse = Vec3::new(0.0, BUMP * 2.0, 0.0);
                    }
                    KeyCode::Numpad5 => {
                        impulse.impulse = Vec3::new(0.0, BUMP * 2.0, 0.0);
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
fn update_countdown_face(
    mut clock_board_face_query: Query<&mut TextMesh, With<ClockBoardFace>>,
    countdown_board: Res<CountdownBoard>,
) {
    if countdown_board.is_running() {
        for mut text in clock_board_face_query.iter_mut() {
            let total_secs = countdown_board.countdown.unwrap().as_secs();
            let minutes = total_secs / 60;
            let seconds = total_secs % 60;
            text.text = format!("{:02}:{:02}", minutes, seconds);
        };
    }
}
fn update_scoreboard(
    mut scoreboard_query: Query<&mut TextMesh, (With<Score>, Without<ClockBoardFace>)>,
    scoreboard: Res<Scoreboard>,
) {
    for mut text in scoreboard_query.iter_mut() {
        text.text = format!("Game Level: {}\nLevel Score: {}\nTotal Score: {}\nToys Left: {}\nBalls Left: {}",
                            scoreboard.level,
                            scoreboard.score, scoreboard.total,
                            scoreboard.toys, scoreboard.balls);
    };
}
fn setup_game_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
    mut scoreboard: ResMut<Scoreboard>,
) {
    let font = asset_server.load("fonts/Archivo.ttf");
    // The countdown clock
    create_countdown_board(&mut commands, &mut meshes, &mut materials, &asset_server);
    // Scoreboard text
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
            base_color: TEXT_COLOR,
            metallic: 0.8,             // Slightly less metallic to show some base color
            perceptual_roughness: 0.3, // Rougher to catch more light highlights
            reflectance: 0.8,
            double_sided: false,
            cull_mode: None,
            ..default()
        })),
        Transform {
            translation: Vec3::new(-14.0, 5.0, 0.0),
            rotation: Quat::from_axis_angle(Vec3::Y, FRAC_PI_2),   // 90 degrees
            scale: Vec3::splat(0.9),
        },
    ));
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
            base_color: TEXT_COLOR, // Blueish metallic
            metallic: 0.8,             // Slightly less metallic to show some base color
            perceptual_roughness: 0.3, // Rougher to catch more light highlights
            reflectance: 0.8,
            double_sided: false,
            cull_mode: None,
            ..default()
        })),
        Transform {
            translation: Vec3::new(-0.0, -1.0, 12.0),
            rotation: Quat::from_axis_angle(Vec3::Y, 0.0),
            scale: Vec3::splat(0.6),
        },
    ));
    // Scoreboard wall
    commands.spawn((
        CollisionGroups::new(FIXED_GROUP, BALL_GROUP | TOY_GROUP),
        RigidBody::Fixed,
        Friction::new(0.5),
        Restitution::new(0.1),
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.5, 8.0, 14.0)))),
        MeshMaterial3d(materials.add(SCOREBOARD_COLOR)),
        Collider::cuboid(0.25, 4.0, 7.0),
        Transform::from_xyz(-14.5, 5.0, 0.0),
    ));

    // Scoring text
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
            base_color: TEXT_COLOR,
            metallic: 0.8,
            perceptual_roughness: 0.3,
            reflectance: 0.8,
            double_sided: false,
            cull_mode: None,
            ..default()
        })),
        Transform {
            translation: Vec3::new(0.0, 2.0, -10.0),
            rotation: Quat::from_axis_angle(Vec3::Y, 0.0),
            scale: Vec3::splat(0.8),
        },
    ));

    // Spawn the Camera
    commands.spawn((
        CameraController{},
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn a Light
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

    // Game Surface, the top of the surface is at y=0.0
    commands.spawn((
        CollisionGroups::new(FIXED_GROUP, BALL_GROUP | TOY_GROUP),
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
                depth: 0.1,
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
            double_sided: false,
            cull_mode: None,
            ..default()
        })),
        Transform {
            translation: Vec3::new(0., 5., -10.0),
            rotation: Quat::from_axis_angle(Vec3::Y, 0.),
            scale: Vec3::new(4.0, 4.0, 2.0),
        },
    ));
}