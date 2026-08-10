use std::{env, thread};

pub fn main() {
    println!("In wasm main");
    start_bevy();    // Never returns until shutdown
}
