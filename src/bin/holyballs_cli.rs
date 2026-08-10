use std::fs::File;
use std::io;
use std::io::{BufReader, Write};
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{sleep, spawn};
use std::time::Duration;
use bevy::prelude::Commands;
use crossfire::mpmc;
use holyballs::*;

pub fn main() {
    let (tx, rx) = mpmc::bounded_async::<ExternalMessage>(3);
    let external_producer = ExternalProducer::new(tx.clone());
    let external_consumer = ExternalConsumer::new(rx);

    let h = spawn(move || {
            command_loop(external_producer);
        }
    );
    start_bevy(external_consumer);
    //    h.join().expect("Command loop failed");
}

// Bevy must be run from main so command loop is spawned.
fn command_loop(external_producer: ExternalProducer) {
    loop {
        print!("hb> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim();
        if input == "exit" {
            external_producer.send(ExternalMessage::new());
            return;
        }
        if input == "start" {
        }
        println!("I got: '{input}'");

        //    let command = input.trim();
    }
}
fn setup_configuration(
    //    mut configuration: ResMut<Configuration>,
    mut commands: Commands,
) {
    // Only good for standalone (testing) so replace with AssetServer
    let path = Path::new("site/assets/config/base.hb.json");
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
