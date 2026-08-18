//! This example demonstrates how to use BSN to compose scenes.

use std::ops::Sub;
use std::panic::Location;
use bevy::prelude::Vec3;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::time::Duration;
use holyballs::*;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_systems(Startup, init_toys)
        .add_systems(Startup, init_camera)
        .add_systems(Update, release_toys)
        .insert_resource(ToyDrop::new(Duration::from_secs(1)))
        .insert_resource(ClearColor(Color::srgb(0.5, 1.0, 0.9))) // Light blue background
        .run();
}
fn init_camera(mut commands: Commands) {
    // Spawn the Camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 25.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Spawn a Light
    commands.spawn((
        //        DirectionalLight::default(),
        PointLight {
            color: Color::from(Color::WHITE),
            shadow_maps_enabled: true,
            intensity: 25_000_000.0,
            range: 80.0,
            radius: 1.0,
            shadow_depth_bias: 0.2,
            ..default()
        },
        Transform::from_xyz(0.0, 20.0, 10.0),
    ));

}
#[derive(Resource)]
struct ToyDrop {
    pace: Duration,
    left: Duration,
    toys: Vec<Transform>,
}
impl ToyDrop {
    fn new(pace: Duration) -> Self {
        Self{pace, left: Duration::ZERO, toys: Vec::new()}
    }

    fn start(&mut self) {
        self.left = self.pace;
    }

    fn add(&mut self, location: Transform) {
        self.toys.push(location);
    }

    fn check_time(&mut self, delay: Duration) -> bool {
        if self.toys.len() <= 0 {
            false
        } else {
//            println!("Left: {}ms", self.left.as_millis());
            if self.left >= delay {
                self.left = self.left.sub(delay);
                false
            } else {
                true
            }
        }
    }

    fn spawn_next(&mut self, mut commands: Commands) -> bool {
        if self.toys.is_empty() {
            false
        } else {
            println!("Release scene");
            let location = self.toys.pop().unwrap();
            commands.spawn_scene(bsn!{
                @ToyBlock {location}
            });
//            let _r = self.spawn_next(commands);
            true
        }
    }
}
fn release_toys(time: Res<Time>, mut toy_drop: ResMut<ToyDrop>, commands: Commands) {
    if toy_drop.check_time(time.delta()) {
        toy_drop.spawn_next(commands);
    }
}
#[derive(SceneComponent, Default, Clone)]
struct ToyBlock {
    transform: Transform,
}

impl ToyBlock {
    fn new(transform: Transform) -> Self {
        Self{transform}
    }
    fn scene() -> impl Scene {
        bsn! {
                template_value(t)
//            CollisionGroups::new(TOY_GROUP, BALL_GROUP | FIXED_GROUP | TOY_GROUP)
//            template_value(RigidBody::Dynamic)
//            Collider::cuboid(0.5, 0.5, 0.5)
//            ToyType { dynamic: true }
//            Friction::new(0.2)
//            Restitution::new(0.1)
            Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
            MeshMaterial3d<StandardMaterial>(asset_value(StandardMaterial {
                base_color: Color::srgb(0.0, 0.0, 1.0),
                ..default()
            }))
 //           ExternalImpulse::default()
//            PointValue::new(15)
        }
    }
}
fn init_toys(
    mut toy_drop: ResMut<ToyDrop>,
) {
    println!("Add block");
    toy_drop.add(ToyBlock{});
    toy_drop.start();
}