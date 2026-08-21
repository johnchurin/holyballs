use bevy::asset::Asset;
use bevy::color::{Color, Srgba};
use bevy::math::Vec3;
use bevy::prelude::{Resource, TypePath};
use serde::{Deserialize, Serialize};
use serde_json::Result;
// Default settings when not specified in configuration
const BACKGROUND_COLOR: Color = Color::srgb(0.7, 0.8, 0.7);
const BARRIER_COLOR: Color = Color::srgb(1.0, 0.64, 0.0);
const FENCE_COLOR: Color = Color::srgba(0.0, 0.9, 0.0, 0.4);
const TABLE_COLOR: Color = Color::srgb(0.0, 1.0, 0.0);
const WALL_COLOR: Color = Color::srgb(173.0/255.0, 216.0/255.0, 230.0/255.0);
const SCOREBOARD_COLOR: Color = Color::srgb(0.8, 0.2, 0.2);

// A list of available games, deserialized from json
#[derive(Default, Clone, Debug)]
#[derive(Serialize, Deserialize, Asset, TypePath)]
pub struct Menus {
    pub title: String,
    pub entries: Vec<MenuItem>,
}
#[derive(Default, Clone, Debug)]
#[derive(Serialize, Deserialize, Asset, TypePath)]
pub struct MenuItem {
    pub name: String,
    pub display: String,
    pub file: String,
}

#[derive(Resource)]
#[derive(Default)]
#[derive(Deserialize, Asset, TypePath)]
pub struct Configuration {
    name: Option<String>,
    _description: Option<String>,
    pace: Option<u32>,
    background_color: Option<String>,
    _title_color: Option<String>,
    table_color: Option<String>,
    barrier_color: Option<String>,
    fence_color: Option<String>,
    wall_color: Option<String>,
    scoreboard_color: Option<String>,
    levels: Vec<GameLevel>,
}
impl Configuration {
    // Create an empty placeholder until a real configuration is loaded
    pub fn new() -> Self {
        Self {
            levels: Vec::new(),
            name: Some("Initial".to_string()),
            ..Configuration::default()
        }
    }
    pub fn from_json_string(json: String) -> Result<Configuration> {
    let config: Result<Configuration> = serde_json::from_str(json.as_str());
        config
    }
    pub fn get_pace(&self) -> Option<u32> {
        self.pace
    }
    pub fn get_background_color(&self) -> Color {
        if self.background_color.is_some() {
            Srgba::hex(self.background_color.as_ref().unwrap()).unwrap().into()
        } else {
            BACKGROUND_COLOR
        }
    }
    pub fn get_table_color(&self) -> Color {
        if self.table_color.is_some() {
            Srgba::hex(self.table_color.as_ref().unwrap()).unwrap().into()
        } else {
            TABLE_COLOR
        }
    }
    pub fn get_barrier_color(&self) -> Color {
        if self.barrier_color.is_some() {
            Srgba::hex(self.barrier_color.as_ref().unwrap()).unwrap().into()
        } else {
            BARRIER_COLOR
        }
    }
    pub fn get_fence_color(&self) -> Color {
        if self.fence_color.is_some() {
            Srgba::hex(self.fence_color.as_ref().unwrap()).unwrap().into()
        } else {
            FENCE_COLOR
        }
    }
    pub fn get_wall_color(&self) -> Color {
        if self.wall_color.is_some() {
            Srgba::hex(self.wall_color.as_ref().unwrap()).unwrap().into()
        } else {
            WALL_COLOR
        }
    }
    pub fn get_scoreboard_color(&self) -> Color {
        if self.scoreboard_color.is_some() {
            Srgba::hex(self.scoreboard_color.as_ref().unwrap()).unwrap().into()
        } else {
            SCOREBOARD_COLOR
        }
    }
    pub fn get_level_count(&self) -> i32 {
        self.levels.len() as i32
    }

    pub fn get_name(&self) -> String {
        if self.name.is_some() {
            self.name.clone().unwrap()
        } else {
            String::from("No gemae name")
        }
    }
    fn _add(&mut self, level: GameLevel) -> &mut Self {
        self.levels.push(level);
        self
    }

    pub fn get_game_level(&self, level: i32) -> Option<&GameLevel> {
        // Level is 1 origin, levels Vec is zero origin, so we return level-1)
        if level > self.levels.len() as i32 {
            None
        } else {
            Some(self.levels.get(level as usize - 1).unwrap())
        }
    }
}
#[derive(Default, Clone, Debug)]
#[derive(Deserialize, Asset, TypePath)]
pub struct GameLevel {
    pub seconds: Option<i32>,
    balls: Option<i32>,
    pub barriers: Option<i32>,
    pub blocks: Option<i32>,
    pub disks: Option<i32>,
    pub cones: Option<i32>,
    pub blacks: Option<i32>,
    pub dips: Option<i32>,
    pub bumpys: Option<i32>,
    pub targets: Option<i32>,
    pub spikeys: Option<i32>,
    pub ghosts: Option<i32>,
    pub lifesavers: Option<i32>,
    pub cylinders: Option<i32>,
    pub fences: Option<i32>,
    pub wind: Option<Vec3>,
    pub help: String,
}
impl GameLevel {
    // If no balls specified, give them three balls
    pub fn get_ball_count(&self) -> i32 {
        if self.balls.is_some() {
            let b = self.balls.unwrap();
            if b < 1 {
                3
            } else { b }
        } else { 3 }
    }
}
