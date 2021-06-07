//! #stock_tracker
//!
//! This is the runtime logic for the rust_stock_tracker project

use std::error::Error; // So we may define Box<dyn Error> // To allow for the use of `env::Args` in setting up `Config`

/// The `Command` enum represents the variety of input cases a user could specify.
pub enum Command {
    Create,
    Delete,
    Login,
    Logout,
    Showall,
    Invalid,
}

impl Command {

    /// Constructor for the `Command` enum to parse a `String` input
    pub fn new(s: &str) -> Command {
        match String::from(s).to_lowercase().as_str() {
            "c" | "create" => Command::Create,
            "d" | "delete" => Command::Delete,
            "li" | "login" => Command::Login,
            "lo" | "logout" => Command::Logout,
            "sa" | "showall" => Command::Showall,
            _ => Command::Invalid,
        }
    }

    /// Returns the number of arguments expected after the `Command`
    pub fn num_args(self) -> i32 {
        match self {
            Command::Create => 1,
            Command::Delete => 1,
            Command::Login => 1,
            Command::Logout => 0,
            Command::Showall => 0,
            Command::Invalid => 0,
        }
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
    pub fn new<Args: Iterator<Item = String>>(mut args: Args) -> Result<Config, &'static str> {
        args.next(); // Discard the first argument

        let command = match args.next() {
            Some(arg) => Command::new(&arg),
            None => return Err("Didn't get a command string"),
        };

        let remainder = args.collect();

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
            Err(_) => true,
        });
    }

    #[test]
    fn config_new_one_arg() {
        assert!(match Config::new(vec![String::from("test1")].into_iter()) {
            Ok(_) => false,
            Err(_) => true,
        });
    }

    #[test]
    fn config_new_two_args() {
        assert!(
            match Config::new(vec![String::from("test1"), String::from("test2")].into_iter()) {
                Ok(Config {command: Command::Invalid, ..}) => true,
                _ => false,
            }
        );
    }

    #[test]
    fn config_new_many_args() {
        let mut check = true;

        for i in 3..100 {
            let mut v = Vec::<String>::new();
            for j in 0..i {
                v.push(format!("test{}", j+1));
            }
            check = check &&
                match Config::new(v.clone().into_iter()) {
                    Ok(Config {command: Command::Invalid, remainder}) => remainder == v[2..],
                    _ => false,
                }
        }

        assert!(check);
    }
}
