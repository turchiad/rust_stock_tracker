//! #stock_tracker
//!
//! This is the runtime logic for the rust_stock_tracker project

use std::error::Error; // So we may define Box<dyn Error> // To allow for the use of `env::Args` in setting up `Config`
use std::fmt; // So we may define `Display` for `Command`


/// The `Command` enum represents the variety of input cases a user could specify.
pub enum Command {
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

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
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
    fn config_new_one_invalid_arg() {
        assert!(match Config::new(vec![String::from("test1")].into_iter()) {
            Ok(_) => false,
            Err(x) => x == "Invalid command string",
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
}
