//! #stock_tracker
//!
//! This is the runtime logic for the rust_stock_tracker project

// std
use std::collections::HashMap; // So we may construct HashMaps of passwords & users
use std::error::Error; // So we may define Box<dyn Error> // To allow for the use of `env::Args` in setting up `Config`
use std::fmt; // So we may define `Display` for `Command`
use std::fs; // So we may read/write to files.
use std::io;
use std::io::Write;
use std::path::Path;

// external crates
// use serde::{Serialize, Deserialize}; // So we may prepare the HashMap to be written to a file
use serde_json; // So we may write and read the HashMap to JSON
// use thiserror::Error; // For more structured definition of errors

// internal crates
use user::User;

// Kinds of errors we expect
// 1. IOError
// 2. 

// #[derive(Error, Debug)]
// pub enum Error {

// }

/// The `Command` enum represents the variety of input cases a user could specify.
pub enum Command {
    Init,
    Create,
    Delete,
    Login,
    Logout,
    Showall,
}

impl Command {

    /// Constructor for the `Command` enum to parse a `String` input
    pub fn new(s: &str) -> Result<Command, &'static str> {
        Ok(match String::from(s).to_lowercase().as_str() {
            "i" | "init" => Command::Init,
            "c" | "create" => Command::Create,
            "d" | "delete" => Command::Delete,
            "li" | "login" => Command::Login,
            "lo" | "logout" => Command::Logout,
            "sa" | "showall" => Command::Showall,
            _ => return Err("Invalid command string"),
        })
    }

    /// Returns the number of arguments expected after the `Command`
    pub fn num_args(&self) -> i32 {
        match self {
            Command::Init => 0,
            Command::Create => 1,
            Command::Delete => 1,
            Command::Login => 1,
            Command::Logout => 0,
            Command::Showall => 0,
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self{
            Command::Init => "init",
            Command::Create => "create",
            Command::Delete => "delete",
            Command::Login => "login",
            Command::Logout => "logout",
            Command::Showall => "showall"
        })
    }
}

/// The `Config` struct represents the CLI input state of a call to this program.
pub struct Config {
    /// The primary command immediately following the call
    pub command: Command,
    /// The remainder of arguments which may be processed differently depending on the command.
    pub remainder: Vec<String>,
}

impl Config {
    pub fn new<Args: Iterator<Item = String>>(mut args: Args) -> Result<Config, String> {
        args.next(); // Discard the first argument

        let command = match args.next() {
            Some(arg) => Command::new(&arg)?, // Return Err if invalid
            None => return Err(String::from("Didn't get a command string")),
        };

        let remainder: Vec<String> = args.collect();

        // Check if valid # of args have been provided
        if (remainder.len() as i32) < command.num_args() {
            return Err(format!("Too few arguments provided for {}", command))
        }

        Ok(Config { command, remainder })
    }
}

/// The `run` function represents the runtime logic of the program
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.command {
        Command::Init => init(config)?,
        Command::Create => create(config)?,
        Command::Delete => delete(config)?,
        Command::Login => login(config)?,
        Command::Logout => logout(config)?,
        Command::Showall => showall(config)?,
    };

    Ok(())
}

//
// User-actuated functions
//

/// The `init` function produces a HashMap at a default location
fn init(_config: Config) -> Result<(), String> {
    let hashmap = HashMap::<String, User>::new();
    write_to_hashmap(&"HashMap.txt", &hashmap)
}

/// The `create` function opens the HashMap and inserts a new user. 
fn create(config: Config) -> Result<(), String> {

    let username = &config.remainder[0];

    let f = |hashmap: &mut HashMap<String, User>| hashmap
        .insert(username.to_string(), User::new()?) // Insert
        .ok_or_else(|| String::from("")).map(|_| ()); // Handle Option -> Result & discarding User

    modify_hashmap(&"HashMap.txt", f)
}

/// The `delete` function queries the user for a confirmation, opens the HashMap, and deletes a user.
fn delete(config: Config) -> Result<(), String> {
    
    let username = &config.remainder[0];

    // Make sure the user wants to delete
    println!("Are you sure you want to delete user profile {}", username.to_string());

    let mut ans = String::new();
    match io::stdin().read_line(&mut ans) {
        Err(x) => return Err(format!("{:?}",x)),
        _ => ..,
    };

    // Remove the newline
    let ans = ans.trim();

    // Debug
    //println!("fc: {}, lc: {}, lc == yes: {}", ans, ans.to_lowercase(), ans.to_lowercase().as_str() == "yes");

    match ans.to_lowercase().as_str() {
        // In the case where the user is sure
        "y" | "yes" => {
            let f = |hashmap: &mut HashMap<String, User>| hashmap
                .remove(&username.to_string()) // Remove
                .ok_or_else(|| String::from("")).map(|_| ()); // Hnandle Option -> Result & discarding User
            modify_hashmap(&"HashMap.txt", f)
        },
        // In the case where the user declines
        "q" | "quit" | "n" | "no" => Ok(()),
        // In the case where the user input is not recognized
        _ => Err(String::from("Input not recognized.")),
    }
}

/// The `login` function queries the user for a password, opens the HashMap, and activates a state where certain commmands will be applied on the user in question.
fn login(config: Config) -> Result<(), Box<dyn Error>>{
    unimplemented!()
}

/// The `logout` function deactivates the state where certain commands will be applied on the user in question.
fn logout(config: Config) -> Result<(), Box<dyn Error>>{
    unimplemented!()
}

/// The `showall` function relies on a logged in state and shows the current state of all the logged in user's stoc<P: AsRef<Path>>ks.
fn showall(config: Config) -> Result<(), Box<dyn Error>>{
    unimplemented!()
}

//
// Assistive functions
//

/// The `read_from_hashmap` function takes a `Path` and returns the `HashMap<String, User>` located at that path
/// using `serde_JSON` to read the file.
fn read_from_hashmap<P: AsRef<Path>>(path: &P) -> Result<HashMap<String, User>, String> {

    let file = match fs::File::open(path) {
        Ok(x) => x,
        Err(_) => return Err(format!("{} has not been initialized in this directory.", path.as_ref().to_str().unwrap()))
    };

    let reader = io::BufReader::new(&file);

    match serde_json::from_reader(reader) {
        Ok(x) => x,
        Err(x) => return Err(format!("{:?}",x)),
    }
}

/// The 'write_to_hashmap` function takes a `Path` and a `HashMap<String, User>` and writes the
/// `HashMap<String, User>` to the file located at that path using `serde_JSON` to write the file.
fn write_to_hashmap<P: AsRef<Path>>(path: &P, hashmap: &HashMap<String, User>) -> Result<(), String> {
    
    let serialized_hashmap = serde_json::to_string(hashmap).unwrap();

    let mut file = match fs::File::create(path) {
        Ok(x) => x,
        Err(_) => return Err(format!("Opening {} write-only failed.", path.as_ref().to_str().unwrap())),
    };

    match file.write_all(serialized_hashmap.as_bytes()) {
        Err(x) => return Err(format!("{:?}",x)),
        _ => ..,
    };

    Ok(())
}

fn modify_hashmap<P, F>(path: &P, f: F) -> Result<(), String> where 
    P: AsRef<Path>,
    F: Fn(&mut HashMap<String, User>) -> Result<(), String> {
    
    let hashmap = &mut read_from_hashmap(path)?;
    f(hashmap)?;
    write_to_hashmap(path, &hashmap)
}
//
// Testing
//

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_new_no_args() {
        assert!(match Config::new(Vec::<String>::new().into_iter()) {
            Ok(_) => false,
            Err(x) => x == "Didn't get a command string",
        });
    }

    #[test]
    fn config_new_one_arg() {
        assert!(match Config::new(vec![String::from("test1")].into_iter()) {
            Ok(_) => false,
            Err(x) => x == "Didn't get a command string",
        });
    }

    #[test]
    fn config_new_two_invalid_args() {
        assert!(
            match Config::new(vec![String::from("test1"), String::from("test2")].into_iter()) {
                Ok(_) => false,
                Err(x) => x == "Invalid command string",
            }
        );
    }

    #[test]
    fn config_new_many_invalid_args() {
        let mut check = true;

        for i in 3..100 {
            let mut v = Vec::<String>::new();
            for j in 0..i {
                v.push(format!("test{}", j+1));
            }
            check = check &&
                match Config::new(v.clone().into_iter()) {
                    Ok(_) => false,
                    Err(x) => x == "Invalid command string",
                }
        }

        assert!(check);
    }

    #[test]
    fn config_new_two_valid_args() {
        assert!(
            match Config::new(vec![String::from("test1"), String::from("showall")].into_iter()) {
                Ok(Config {command: Command::Showall, ..}) => true,
                _ => false,
            }
        );
    }

    #[test]
    fn config_new_two_valid_args_invalid_num_args() {
        assert!(
            match Config::new(vec![String::from("test1"), String::from("create")].into_iter()) {
                Ok(_) => false,
                Err(x) => x == "Too few arguments provided for create",
            }
        );
    }    

    #[test]
    fn config_new_three_valid_args() {
        assert!(
            match Config::new(vec![String::from("test1"), String::from("create"), String::from("test3")].into_iter()) {
                Ok(Config {
                    command: Command::Create,
                    remainder,
                }) => remainder == vec![String::from("test3")],
                _ => false,
            }
        );
    }
}