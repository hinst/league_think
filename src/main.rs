use std::env;

mod store;
mod analyze;

pub fn main() {
    println!("STARTING...");
    let args: Vec<String> = env::args().collect();
    let command = args.get(1);
    match command {
        Some(command) => {
            if command == "store" {
                store::store();
            } else if command == "analyze" {
                analyze::analyze();
            } else {
                println!("Unknown command: {}", command);
            }
        }
        _ => {
            println!("Please provide a command");
        }
    }
}