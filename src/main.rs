use std::path::Path;

mod gtfs;
mod network;
mod text_interface;
mod geo_utils;

use network::*;
use text_interface::*;

fn main() {
    println!("Hello, world! Prahadlo here!");
    let nw = Network::new(Path::new("data/"));
    nw.print_debug_info();
    loop {
        process_command(&nw);
    }
}
