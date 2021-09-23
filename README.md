# Prahadlo
Prahadlo is a command-line tool for quickly finding public transport connections in Prague written in Rust.
It was created as a student project for the Programming 1 course at [MFF UK](https://www.mff.cuni.cz/).
It works with the GTFS dataset published by PID at the [PID Opendata](https://pid.cz/o-systemu/opendata/) website.

## Setup
Install `Rust` if you haven't already. The easiest way is probably through `rustup`.
For a detailed guide, please refer to the [official Rust website](https://www.rust-lang.org/tools/install).

Then, you need to download the [Prague GTFS dataset](http://data.pid.cz/PID_GTFS.zip) and extract it into a folder
called `data/` in the project root.

Compile and run the project using `cargo run`.

## Usage
DISCLAIMER - basically all commands other than `conn` are for debug. I kept them in the interface
for everyone interested in the internal representation in the program.

After running `cargo run`, you will be greeted with a prompt. The commands for the prompt are:
  * `conn [time] | [stop_name_1] | [stop_name_2]`
  prints the shortest connection from [stop n.1] to [stop n.2] at the time provided. 
  * `help`
  prints a help message
  * `stop [stop_id]`
  prints information about a specific node. A node is an internal data structure. [stop_id] = string
  * `node [node_id]`
  prints information about a specific node. A node is an internal data structure. [node_id] = unsigned integer
  * `trip [trip_id]`
  prints information about a specific GTFS trip. [trip_id] = string

## Future plans
As this was a semester project, there were a lot of things that I would like to do but didn't manage to implement
them in time. These include:
    * a command to output the time from one stop to all other stops at given time to a CSV. This would be useful for example for heatmaps of Prague
    * loading transfers from the GTFS dataset
    * replace the in-house module `gtfs.rs` with a specialized library, for example [`gtfs-structure`](https://github.com/rust-transit/gtfs-structure)

## Thanks
I would like to thank Martin Mareš and Jirka Škrobánek for the great programming course that taught me a lot.
I would also like to thank the creators of [KSP](ksp.mff.cuni.cz) and especially Filip Štědronský, who wrote
the serial about finding connections in public transport. It is available [here](http://ksp.mff.cuni.cz/h/ulohy/32/zadani3.html#task-32-3-6), go take a look! It is a very good read.