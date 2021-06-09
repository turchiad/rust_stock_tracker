//! #stock_tracker
//!
//! This is the runtime logic for the rust_stock_tracker project

// std
use std::io::Write;
use std::io::BufReader;
use std::collections::HashMap; // So we may construct HashMaps of passwords & users
use std::error::Error; // So we may define Box<dyn Error> // To allow for the use of `env::Args` in setting up `Config`
use std::fmt; // So we may define `Display` for `Command`
use std::fs; // So we may read/write to files.

// external crates
use rpassword; // So we may prompt the user for a password without showing their input
use serde::{Serialize, Deserialize}; // So we may prepare the HashMap to be written to a file
use serde_json;

// internal crates
use user;

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

/// The `init` function produces a HashMap at a default location
fn init(_config: Config) -> Result<(), Box<dyn Error>> {
    let hash = HashMap::<String,user::User>::new();

    let serialized_hash = serde_json::to_string(&hash).unwrap();

    let mut file = fs::File::create("HashMap.txt")?;

    file.write_all(serialized_hash.as_bytes())?;

    Ok(())
}

/// The `create` function queries the user for a password, opens the HashMap and inserts a new user. 
fn create(_config: Config) -> Result<(), String> {

    let password = String::from("test");

    let file = match fs::File::open("HashMap.txt") {
        Ok(x) => x,
        Err(_) => return Err(String::from("HashMap.txt has not been initialized in this directory."))
    };


    let reader = BufReader::new(&file);

    let mut hash: HashMap::<String, user::User> = match serde_json::from_reader(reader) {
        Ok(x) => x,
        Err(x) => return Err(format!("{:?}",x)),
    };

    hash.insert(password, user::User::new()?);

    let serialized_hash = serde_json::to_string(&hash).unwrap();

    let mut file = match fs::File::create("HashMap.txt") {
        Ok(x) => x,
        Err(_) => return Err(String::from("Opening HashMap.txt write-only failed."))
    };

    match file.write_all(serialized_hash.as_bytes()) {
        Err(x) => return Err(format!("{:?}",x)),
        _ => ..,
    };

    Ok(())
}

/// The `delete` function queries the user for a confirmation, opens the HashMap, and deletes a user.
fn delete(config: Config) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

/// The `login` function queries the user for a password, opens the HashMap, and activates a state where certain commmands will be applied on the user in question.
fn login(config: Config) -> Result<(), Box<dyn Error>>{
    unimplemented!()
}

/// The `logout` function deactivates the state where certain commands will be applied on the user in question.
fn logout(config: Config) -> Result<(), Box<dyn Error>>{
    unimplemented!()
}

/// The `showall` function relies on a logged in state and shows the current state of all the logged in user's stocks.
fn showall(config: Config) -> Result<(), Box<dyn Error>>{
    unimplemented!()
}

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