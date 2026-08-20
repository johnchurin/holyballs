use std::{fs, io};
use std::io::{Write};
use std::path::{Path, PathBuf};
use std::thread::{spawn};
use crossfire::{mpmc, MAsyncTx};
use holyballs::*;
use std::fs::File;
use std::io::{BufReader};
use std::sync::OnceLock;
use crossfire::mpmc::Array;
use holyballs::config::{MenuItem, Menus};

const CONFIG_DIR: &str = "config";
static TX: OnceLock<MAsyncTx<Array<ExternalMessage>>> = OnceLock::new();

pub fn main() {
    let (tx, rx) = mpmc::bounded_async::<ExternalMessage>(3);
    let external_producer = ExternalProducer::new(tx.clone());

    let external_consumer = ExternalConsumer::new(rx);
    let external_reply = ExternalReply::new(reply_handler);
    TX.get_or_init(|| {
        tx.clone()
    });

    let _h = spawn(move || {
            command_loop(external_producer);
        }
    );
    start_bevy(external_consumer, external_reply);
}

fn reply_handler(message: ExternalMessage)
{
    let tx_handle = TX.get();
    if tx_handle.is_some() {
        let tx = tx_handle.unwrap();

        match message.action.as_str() {
            "game_ended" => {
                let message = ExternalMessage { action: "exit".to_string(), payload: None };

                tx.try_send(message).expect("No tx channel");
            }
            _ => {
                println!("Invalid reply message from game: {:?}", message.action);
            }
        }
    }
}

// Bevy must be run from main thread so command loop is spawned.
fn command_loop(external_producer: ExternalProducer) {
    let m = get_menu();
    if m.is_err() {
        println!("Error opening menu file: {:?}", m.err());
        return;
    }
    println!("Menu choices:");
    let menus = &m.unwrap().entries;
    for menu in menus {
        println!("\t{}, {}, file: {}", menu.name, menu.display, menu.file);
    }
    println!();
    let mut game_loaded = false;
    loop {
        print!("holyballs> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");
        let args: Vec<&str> = input.split_whitespace().collect();
        if args.is_empty() {
            continue;
        }
        let cmd = args[0].to_lowercase();
        if cmd == "exit" {
            external_producer.send(ExternalMessage::new(String::from("exit"), None));
            return;
        }
        if cmd == "fullscreen" {
            let mode = if args.len() > 1 {
                Some(args[1].to_string())
            } else {
                Some("on".to_string())
            };
            external_producer.send(ExternalMessage::new(String::from("fullscreen"), mode));
            continue;
        }
        if cmd == "play" {
            if game_loaded {
                external_producer.send(ExternalMessage::new(String::from("play"), None));
            } else {
                println!("Load a game first");
            }
            continue;
        }

        if cmd == "sound" {
            let payload = if args.len() == 2 {Some(String::from(args[1]))} else {None};
            external_producer.send(ExternalMessage::new(String::from("sound"), payload));
            continue;
        }

        if cmd == "load" {
            let name = if args.len() == 2 {
                String::from(args[1])
            } else {
                String::from("beginner")
            };
            let mut m: Option<&MenuItem> = None;
            for menu in menus {
                if menu.name == name {
                    m = Some(menu);
                    break;
                }
            }
            if m.is_some() {
                let menu = m.unwrap();
                let pathbuf = resolve_path(Path::new(menu.file.as_str()));
                println!{"File: {:?}", pathbuf};
                let json = fs::read_to_string(pathbuf);
                if json.is_err() {
                    println!("Error opening menu file {:?}", json.err());
                    continue;
                }
                external_producer.send(ExternalMessage{action: "load".to_string(), payload: Some(json.unwrap())});
                game_loaded = true;
            } else {
                println!("{} not found in menu", name);
            }
            continue;
        }
        println!("Unknown command: '{cmd}'");
    }
}

fn get_menu() -> Result<Menus, String> {
    let pathbuf = resolve_path(Path::new("menu.json"));
    let file = File::open(pathbuf);
    if file.is_err() {
        return Err("Error opening menu file".to_string());
    }
    let r: serde_json::Result<Menus> = serde_json::from_reader(BufReader::new(file.unwrap()));
    if r.is_ok() {
        Ok(r.unwrap())
    } else {
        println!("Error: {:?}", r.err());
        Err("Parse error:".to_string())
    }
}

fn resolve_path(user_path: &Path) -> PathBuf {
    if user_path.is_absolute() {
        user_path.to_path_buf()
    } else {
        let default_dir = Path::new(CONFIG_DIR);
        default_dir.join(user_path)
    }
}