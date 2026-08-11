use std::{fs, io};
use std::io::{Write};
use std::path::Path;
use std::thread::{spawn};
use crossfire::mpmc;
use holyballs::*;

pub fn main() {
    let (tx, rx) = mpmc::bounded_async::<ExternalMessage>(3);
    let external_producer = ExternalProducer::new(tx.clone());
    let external_consumer = ExternalConsumer::new(rx);

    let _h = spawn(move || {
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

        if args[0].eq_ignore_ascii_case("sound") {
            let payload = if args.len() == 2 {Some(String::from(args[1]))} else {None};
            external_producer.send(ExternalMessage::new(String::from("sound"), payload))
        }

        if args[0].eq_ignore_ascii_case("load") {
            let filename = if args.len() == 2 {
                String::from(args[1])
            } else {
                String::from("assets/config/base.hb.json")
            };
            let path = Path::new(filename.as_str());
            println!("Loading {}", path.display());
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
