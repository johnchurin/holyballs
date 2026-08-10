use std::fs::File;
use std::{fs, io};
use std::io::{BufReader, Read, Write};
use std::path::Path;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{sleep, spawn};
use std::time::Duration;
use bevy::prelude::Commands;
use crossfire::mpmc;
use serde_json::from_reader;
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

// Bevy must be run from main thread so command loop is spawned.
fn command_loop(external_producer: ExternalProducer) {
    loop {
        print!("holyballs> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let args: Vec<&str> = input.split_whitespace().collect();
        if args.is_empty() {
            continue;
        }
        if args[0].eq_ignore_ascii_case("exit") {
            external_producer.send(ExternalMessage::new(String::from("exit"), None));
            return;
        }
        if args[0].eq_ignore_ascii_case("play") {
            external_producer.send(ExternalMessage::new(String::from("play"), None))
        }

        if args[0].eq_ignore_ascii_case("load") {
            if args.len() != 2 {
                println!("Err: Specify filename");
                continue;
            }
            let path = Path::new(args[1]);
            let json = fs::read_to_string(path);
            if json.is_err() {
                println!("Error opening cnfiguration file");
                continue;
            }
            external_producer.send(ExternalMessage{action: "load".to_string(), payload: Some(json.unwrap())});
            continue;
        }
        println!("I got: '{input}'");
        //    let command = input.trim();
    }
}
