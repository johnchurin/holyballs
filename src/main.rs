
use bevy::audio::Volume;
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use rand::RngExt;

const DEAD_BALL: Color = Color::srgb(0.9, 0.0, 0.9);
const LIVE_BALL: Color = Color::srgb(1.0, 0.0, 0.0);
const LIGHT_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const _FENCE_COLOR: Color = Color::srgb(0.0, 0.0, 1.0);
const FLOOR_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);
const CYLINDER_COLOR: Color = Color::srgb(1.0, 1.0, 0.0);
const _CYLINDER_HALF_HEIGHT: f32 = 2.0;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        // Initialize the Rapier physics engine and the debug renderer
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
//        .add_plugins(RapierDebugRenderPlugin::default())
        .add_systems(Startup, setup_physics)
        .add_systems(Update, cleanup_dead_balls)
        .insert_resource(Selected{ entity: None, prev_entity: None })
        .insert_resource(ScoreBoard::new())
        .insert_resource(GlobalVolume::new(Volume::Linear(0.25)))
        .insert_resource(PostSize{radius:1.0,height:4.0})
        .add_systems(Update, (
            drop_a_ball.run_if(input_just_pressed(KeyCode::Enter)),
            drop_a_ball.run_if(input_just_pressed(KeyCode::NumpadEnter)),
            impulse_up.run_if(input_just_pressed(KeyCode::Space)),
            impulse_up.run_if(input_just_pressed(KeyCode::Numpad5)),
            impulse_left.run_if(input_just_pressed(KeyCode::ArrowLeft)),
            impulse_left.run_if(input_just_pressed(KeyCode::Numpad4)),
            impulse_right.run_if(input_just_pressed(KeyCode::ArrowRight)),
            impulse_right.run_if(input_just_pressed(KeyCode::Numpad6)),
            impulse_back.run_if(input_just_pressed(KeyCode::ArrowDown)),
            impulse_back.run_if(input_just_pressed(KeyCode::Numpad2)),
            impulse_forward.run_if(input_just_pressed(KeyCode::ArrowUp)),
            impulse_forward.run_if(input_just_pressed(KeyCode::Numpad8)),
            box_size.run_if(input_just_pressed(KeyCode::KeyB)),
            cylinder_size.run_if(input_just_pressed(KeyCode::KeyC)),
            handle_sensor_events,
            propagate_color,
            update_score.run_if(resource_changed::<ScoreBoard>),
        )
    )
    .add_observer(handle_cylinder_events)
    .add_observer(un_select)
    .run();
}
#[derive(Component)]
struct Fence {
}

#[derive(Component)]
struct Post {
}
#[derive(Resource)]
struct PostSize {
    radius: f32,
    height: f32,
}

#[derive(Component, Clone, Copy, Debug)]
struct BouncyBall {
    color: Color,
}

#[derive(Component)]
struct BouncyBallChild {
}

#[derive(Component)]
struct SensorChild {
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
    fn hit(&mut self) {
        self.score += 1;
        self.total += 1;
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
#[derive(Event)]
struct Unselect {
}
#[derive(Event)]
struct CylinderEvent {
}

#[derive(Resource)]
struct Selected {
    entity: Option<Entity>,
    prev_entity: Option<Entity>,
}

fn handle_sensor_events(
    mut messages: ResMut<Messages<CollisionEvent>>,
    mut scoreboard: ResMut<ScoreBoard>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    for event in messages.drain() {
//        println!("handle_sensor_events");
        match event {
            CollisionEvent::Started(_entity1, _entity2, _flags) => {
                scoreboard.hit();
                // Create an entity dedicated to playing our background music
                commands.spawn((
                    AudioPlayer::new(asset_server.load("audio/beep.ogg")),
                    PlaybackSettings::ONCE,
//                println!("Something entered the sensor: {:?} and {:?}", entity1, entity2);
                ));}
            CollisionEvent::Stopped(_entity1, _entity2, _flags) => {
//                println!("Something left the sensor: {:?} and {:?}", entity1, entity2);
            }
        }
}
}

fn un_select(
    _event: On<Unselect>,
    mut query: Query<(&mut BouncyBall, Entity), With<BouncyBall>>,
    mut selected : ResMut<Selected>,
) {
    for (mut bouncy_ball, entity) in &mut query {
        if selected.prev_entity == Some(entity) {
            bouncy_ball.color = DEAD_BALL;
            selected.prev_entity = None;
            break;
        }
    }
}
fn cleanup_dead_balls(
    mut commands: Commands,
    mut old_balls: Query<(Entity, &mut Transform), With<BouncyBall>>,
    mut selected : ResMut<Selected>,
) {
    // cleanup old balls out of range
    for (entity, ball) in old_balls.iter_mut() {
        if ball.translation.y < -20.0 {
            if selected.entity == Some(entity) {
               selected.entity = None;
            }
            commands.entity(entity).despawn();
//            println!("Ball despawned");
        }
    }
}

fn drop_a_ball(
    mut old_balls: Query<Entity, With<BouncyBall>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut selected : ResMut<Selected>,
    mut scoreboard: ResMut<ScoreBoard>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    let mut rng = rand::rng();
    let x_pos: f32 = rng.random_range(-12.0..0.);
    let y_pos: f32 = rng.random_range(15.0..25.);
    let z_pos: f32 = rng.random_range(-9.0..9.0);
    // 4. Spawn a Dynamic Bouncing Ball
    let bouncy_ball = BouncyBall {color: Color::from(LIVE_BALL)};

    if keyboard_input.pressed(KeyCode::ShiftLeft)
        || keyboard_input.pressed(KeyCode::ShiftRight)
    {
        scoreboard.reset();
        selected.entity = None;
        selected.prev_entity = None;
        for entity in old_balls.iter_mut() {
            commands.entity(entity).despawn();
        }
        return;
    } else {
        scoreboard.new_round();
    }
    let entity = commands.spawn((
        bouncy_ball,
        RigidBody::Dynamic,
        // Lower the Damping for a more advanced game
        Damping {
            linear_damping: 0.2,
            angular_damping: 0.2,
        },
        Collider::ball(0.5),
        // Adding restitution makes the ball bounce
        Restitution::new(1.0),
//        GravityScale(2.0),
        ExternalImpulse::default(),
        Transform::from_xyz(x_pos, y_pos, z_pos),
        Velocity::linear(Vec3::new(2.0, 0.0, 0.0)),
        Visibility::default(),
    )).with_children(|parent| {
        parent.spawn((
                BouncyBallChild{},
                Mesh3d(meshes.add(Mesh::from(Sphere::new(0.5)))),
                MeshMaterial3d(materials.add(bouncy_ball.color)),
            ));
        }).id();
    selected.prev_entity = selected.entity;
    selected.entity = Some(entity);
    commands.trigger(Unselect{});
}
fn propagate_color(
    parent_query: Query<(&BouncyBall, &Children), Changed<BouncyBall>>,
    mut child_query: Query<&MeshMaterial3d<StandardMaterial>, With<BouncyBallChild>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (bouncy_ball, children) in &parent_query {
        // println!("propagate_color {:?}", bouncy_ball.color);
        for &child in children {
            // println!("propagate_color: in child {}", child);
            let material_handle= child_query.get_mut(child).unwrap();
            // Assign the parent color to the child
            let mut material = materials.get_mut(material_handle).unwrap();
            material.base_color = bouncy_ball.color;
            //    println!("propagate_color set to: {:?} in {:?}", material.base_color, entity);
        }
    }
}
fn box_size(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    query: Single<(&mut Transform, &mut Visibility, Entity), With<Fence>>,
    without_query: Query<&Children, Without<ColliderDisabled>>,
    with_query: Query<Entity, With<ColliderDisabled>>,
    impulses: Query<&mut ExternalImpulse, With<BouncyBall>>,
) {
    let going_up = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
    let increment = if going_up { 0.5 } else { -0.5 };
    let (mut transform, mut visibility, parent_entity) = query.into_inner();
//    println!("Box: Scale before {}", transform.scale);
    if going_up {
        for mut impulse in impulses {
            impulse.impulse = Vec3::new(0.0, 0.1, 0.0);
        }
        //        println!("A little push");
    }
    if transform.scale.y + increment <= 0.0 {
//        println!("Collider too small...");
        for entity in without_query.iter_descendants(parent_entity) {
//            println!("Disabling Collider{:?}", entity);
            commands.entity(entity).insert(ColliderDisabled);
        }
        *visibility = Visibility::Hidden;
    } else {
        for entity in with_query.iter() {
//            println!("Enabling Collider{:?}", entity);
            commands.entity(entity).remove::<ColliderDisabled>();
        }
        *visibility = Visibility::Visible;
        transform.scale.y += increment;
    }
//    println!("Box: Scale after {}", transform.scale);
}
fn handle_cylinder_events(
    _event: On<CylinderEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    post_size: ResMut<PostSize>,
    mut commands: Commands,
) {
//    println!("Spawning new cylinder");
    let mesh_handle = meshes.add(Cylinder::new( post_size.radius, post_size.height));
    commands.spawn((
        Post {},
        RigidBody::Fixed,
        Mesh3d(mesh_handle.clone()),
        MeshMaterial3d(materials.add(CYLINDER_COLOR)),
        Collider::from_bevy_mesh(
            meshes.get(&mesh_handle).unwrap(),
            &ComputedColliderShape::ConvexHull).unwrap(),
//        AsyncSceneCollider::default(),
        Transform::from_xyz(0.0, post_size.height/2.0, 0.0),
        GlobalTransform::from_xyz(0.0, post_size.height/2.0, 0.0),
    )).with_children(|parent| {
        parent.spawn( (
            SensorChild{},
            Collider::cylinder(post_size.height/2.0, post_size.radius),
            Sensor,
            ActiveEvents::COLLISION_EVENTS,
            //            Transform::from_xyz(0.0, 0.0, 0.0),
        ));
    });

}

fn cylinder_size(
    mut query: Query<Entity, With<Post>>,
    mut post_size: ResMut<PostSize>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    impulses: Query<&mut ExternalImpulse, With<BouncyBall>>,
//    selected: Res<Selected>,
) {
    // Remove the previous cylinder
    for entity in query.iter_mut() {
//        println!("Despawned previous");
        commands.entity(entity).despawn();
    }
    let going_up = keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
    let increment = if going_up {0.5} else {-0.5};
    // If we're already too small and we not getting larger, noop.
    if post_size.height <= 0.0 && !going_up{
//        println!("Already too small");
        return;
    }
    // Apply increment
    post_size.height += increment;
    // If the new height is zero, noop.
    if post_size.height <=0.0 {
//        println!("Now too small {:?} to render", post_size.height);
        return;
    }
    // Need a little push on the ball when adding a fresh cylinder underneath it
    if going_up {
        for mut impulse in impulses {
            impulse.impulse = Vec3::new(0.0, 0.1, 0.0);
        }
//        println!("A little push");
    }
    commands.trigger(CylinderEvent{});
}

fn impulse_up(
    mut impulses: Query<&mut ExternalImpulse, With<BouncyBall>>,
    selected: Res<Selected>,
) {
    if selected.entity.is_some() {
        let mut impulse = impulses.get_mut(selected.entity.unwrap()).unwrap();
        impulse.impulse = Vec3::new(0.0, 6.0, 0.0);
    }
}

fn impulse_forward(
    mut impulses: Query<&mut ExternalImpulse, With<BouncyBall>>,
    selected: ResMut<Selected>,
) {
    if selected.entity.is_some() {
        let mut impulse = impulses.get_mut(selected.entity.unwrap()).unwrap();
        impulse.impulse = Vec3::new(0.0, 0.0, -2.5);
    }
}
fn impulse_left(
    mut impulses: Query<&mut ExternalImpulse, With<BouncyBall>>,
    selected: ResMut<Selected>,
) {
    if selected.entity.is_some() {
        let mut impulse = impulses.get_mut(selected.entity.unwrap()).unwrap();
        impulse.impulse = Vec3::new(-2.5, 0.0, 0.0);
    }
}
fn impulse_back(
    mut impulses: Query<&mut ExternalImpulse, With<BouncyBall>>,
    selected: ResMut<Selected>,
) {
    if selected.entity.is_some() {
        let mut impulse = impulses.get_mut(selected.entity.unwrap()).unwrap();
        impulse.impulse = Vec3::new(0.0, 0.0, 2.5);
    }
}

fn impulse_right(
    mut impulses: Query<&mut ExternalImpulse, With<BouncyBall>>,
    selected: ResMut<Selected>,
) {
    if selected.entity.is_some() {
        let mut impulse = impulses.get_mut(selected.entity.unwrap()).unwrap();
        impulse.impulse = Vec3::new(2.5, 0.0, 0.0);
    }
}

fn update_score (
    mut query: Query<&mut Text, With<Text>>,
    scoreboard: Res<ScoreBoard>,
) {
    for mut text in query.iter_mut() {
        if scoreboard.round == 0 {
            *text = Text::new("Press enter to drop a ball");
        } else {
            *text = Text::new(format!("Round: {}\n\nScore: {}\n\nTotal: {}", scoreboard.round, scoreboard.score, scoreboard.total));
        }
    }
}

fn setup_physics(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>)
{

    // Scoreboard
    commands.spawn((
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..Default::default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(12),
            left: px(12),
            ..default()
        },
    ));

    // 1. Spawn the Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // 2. Spawn a Light
    commands.spawn((
//        DirectionalLight::default(),
        PointLight {
            color: Color::from(LIGHT_COLOR),
            shadow_maps_enabled: true,
            intensity: 10_000_000.,
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
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(25.0, 0.5, 20.0)))),
        MeshMaterial3d(materials.add(FLOOR_COLOR)),
        Collider::cuboid(12.5, 0.25, 10.0),
        Transform::from_xyz(2.0, -0.25, 0.0),
    ));

    // Add fence
    commands.spawn( (
        Fence {},
        WorldAssetRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/cube_with_hole.glb#Fence"))),
        AsyncSceneCollider::default(),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    commands.trigger(CylinderEvent{});

}
