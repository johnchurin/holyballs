use std::fs::File;
use std::io;
use std::io::{BufReader, Write};
use std::path::Path;
use bevy::prelude::Commands;
use holyballs::*;
pub fn main() {
    start_bevy();
    loop {
        print!("hb> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        println!("I got: {input}");

    //    let command = input.trim();
    }
}
fn setup_configuration(
    //    mut configuration: ResMut<Configuration>,
    mut commands: Commands,
) {
    // Only good for standalone (testing) so replace with AssetServer
    let path = Path::new("../site/base.hb.json");
    let file = File::open(path).expect("File open error");
    let reader = BufReader::new(file);
    let config: serde_json::Result<Configuration> = serde_json::from_reader(reader);

    commands.insert_resource(config.unwrap());
    println!("Config file read and new config resource inserted");
    // Serialize it to a JSON string.
    // let j = serde_json::to_string_pretty(configuration.as_ref()).unwrap();
    // let filename = "base.hb.json";
    // fs::write(filename, j).expect("Error writing to json file");
    //
    // // Print, write to a file, or send to an HTTP server.
    // println!("File {} created", filename);
}
