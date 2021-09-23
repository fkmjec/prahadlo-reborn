use std::path::Path;

mod gtfs;
mod network;
mod str_utils;
mod text_interface;
mod geo_utils;

use network::*;
use text_interface::*;

fn main() {
    println!("Hello, world! Prahadlo here!");
    let nw = Network::new(Path::new("data/"));
    let mut interface = TextInterface::new("history.txt");
    nw.print_debug_info();
    loop {
        interface.process_command(&nw);
    }
}
