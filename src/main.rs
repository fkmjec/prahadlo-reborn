use std::path::Path;

mod gtfs;
mod network;
mod text_interface;

use network::*;

fn main() {
    println!("Hello, world! Prahadlo here!");
    let nw = Network::new(Path::new("data/"));
    nw.print_debug_info();
    while true {
        let command = text_interface::get_command();
        println!("{:?}", command);
    }
}
