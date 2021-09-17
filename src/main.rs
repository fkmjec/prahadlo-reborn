use std::path::Path;

mod gtfs;
mod network;

use network::*;

fn main() {
    println!("Hello, world! Prahadlo here!");
    let nw = Network::new(Path::new("data/"));
    nw.print_debug_info();
}
