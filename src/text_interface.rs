#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Invalid,
    PrintNode(usize),
    PrintStop(String),
    PrintTrip(String),
}

fn parse_print_node(args: &[&str]) -> Command {
    if args.len() == 1 {
        let node_id = args[0].parse::<usize>();
        match node_id {
            Ok(id) => Command::PrintNode(id),
            Err(_) => Command::Invalid,
        }
    } else {
        Command::Invalid
    }
}

fn parse_print_stop(args: &[&str]) -> Command {
    if args.len() == 1 {
        let stop_id = args[0];
        Command::PrintStop(String::from(stop_id))
    } else {
        Command::Invalid
    }
}

fn parse_print_trip(args: &[&str]) -> Command {
    if args.len() == 1 {
        let trip_id = args[0];
        Command::PrintTrip(String::from(trip_id))
    } else {
        Command::Invalid
    }
}

fn command_from_line(line: &str) -> Command {
    let args: Vec<&str> = line.trim().split(" ").collect();

    // TODO: add validation of command arguments
    match args[0] {
        "node" => parse_print_node(&args[1..]),
        "stop" => parse_print_stop(&args[1..]),
        "trip" => parse_print_trip(&args[1..]),
        "help" => Command::Help,
        _ => Command::Invalid,
    }
}

fn print_help() {
    println!("Commands:");
    println!(" - node [node_id] - prints information about a node with the id");
    println!(" - stop [stop_id] - prints information about a stop with the id");
    println!(" - trip [trip_id] - prints information about a trip with the id");
}

fn print_invalid() {
    
}

pub fn get_command() -> Command {
    let mut line = String::new();
    println!("Enter the command: ");
    std::io::stdin().read_line(&mut line).unwrap();
    let mut command = command_from_line(&line);
    while command == Command::Invalid || command == Command::Help {
        if command == Command::Invalid {
            println!("The command you entered was incorrect!");
        } else {
            print_help();
        }
        let mut line = String::new();
        println!("Enter the command :");
        std::io::stdin().read_line(&mut line).unwrap();    
        command = command_from_line(&line);
    }
    command
}