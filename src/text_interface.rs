#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    PrintNode(usize),
    PrintStop(String),
    PrintTrip(String),
}

fn command_from_line(line: &str) -> Command {
    let args: Vec<&str> = line.trim().split(" ").collect();

    // TODO: add validation of command arguments
    match args[0] {
        "node" => Command::PrintNode(args[1].parse::<usize>().unwrap()),
        "stop" => Command::PrintStop(String::from(args[1])),
        "trip" => Command::PrintTrip(String::from(args[1])),
        _ => Command::Help,
    }
}

fn print_help() {
    println!("Commands:");
    println!("node [node_id] - prints information about a node with the id");
    println!("stop [stop_id] - prints information about a stop with the id");
    println!("trip [trip_id] - prints information about a trip with the id");
}

pub fn get_command() -> Command {
    let mut line = String::new();
    println!("Enter the command :");
    let b1 = std::io::stdin().read_line(&mut line).unwrap();
    let mut command = command_from_line(&line);
    while command == Command::Help {
        print_help();
        let mut line = String::new();
        println!("Enter the command :");
        let b1 = std::io::stdin().read_line(&mut line).unwrap();    
        command = command_from_line(&line);
    }
    command
}