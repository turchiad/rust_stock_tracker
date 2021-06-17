//! #stock_tracker
//!
//! This is the runtime logic for the rust_stock_tracker project

// features
#![feature(map_try_insert)]

// std
use std::collections::HashMap; // So we may construct HashMaps of passwords & users
use std::error::Error; // So we may define Box<dyn Error> // To allow for the use of `env::Args` in setting up `Config`
use std::env; // So we can set the configuration path by environment variables
use std::fmt; // So we may define `Display` for `Command`
use std::fs; // So we may read/write to files.
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;


// external crates
use dirs;
// use serde::{Serialize, Deserialize}; // So we may prepare the HashMap to be written to a file
use serde_json; // So we may write and read the HashMap to JSON
use thiserror::Error; // For more structured definition of errors

// internal crates
use user::User;
use ProjectError::*; // To increse readability

/// The `ProjectError` enum represents the variants of `Error`s expected in `stock_tracker`
#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("Read from HashMap file {} unsuccessful", .0.display())]
    IOHashMapOpenError(PathBuf),
    #[error("Write to HashMap file at {} unsuccessful", .0.display())]
    IOHashMapWriteError(PathBuf),
    #[error("Serialization unsuccessful")]
    SerializeJSONError,
    #[error("Deserialization of JSON file {} unsuccessful", .0.display())]
    DeserializeJSONError(PathBuf),
    #[error("Insertion to HashMap failed: key {0} is already occupied.")]
    HashMapInsertError(String),
    #[error("Remove from HashMap at key {0} unsuccessful")]
    HashMapRemoveError(String),
    #[error("Error creating new User")]
    UserNewError,
    #[error("No command string provided.")]
    ConfigNoCommandError,
    #[error("Too few arguments provided for {0}")]
    ConfigArgumentsError(String),
    #[error("Creation of directories to {} unsuccessful", .0.display())]
    ConfigCreateDirectoryError(PathBuf),
    #[error("Unexpected error: home directory not found. Consider specifying a configuration directory by setting \"RUST_STOCK_TRACKER_CONFIGURATION_DIRECTORY\"")]
    ConfigHomeDirectoryNotFoundError,
    #[error("Command string not recognized.")]
    CommandInvalidError,
    #[error("Input not recognized.")]
    InvalidInputError,
}

/// The `Command` enum represents the variety of input cases a user could specify.
#[derive(Debug)]
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
    pub fn new(s: &str) -> Result<Command, ProjectError> {
        Ok(match String::from(s).to_lowercase().as_str() {
            "i" | "init" => Command::Init,
            "c" | "create" => Command::Create,
            "d" | "delete" => Command::Delete,
            "li" | "login" => Command::Login,
            "lo" | "logout" => Command::Logout,
            "sa" | "showall" => Command::Showall,
            _ => return Err(CommandInvalidError),
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
    /// The location of the program's configuration files
    pub configuration_directory: PathBuf, 
}

impl Config {
    pub fn new<Args: Iterator<Item = String>>(mut args: Args) -> Result<Config, ProjectError> {
        args.next(); // Discard the first argument

        // command
        let command = match args.next() {
            Some(arg) => Command::new(&arg)?, // Return Err if invalid
            None => return Err(ConfigNoCommandError),
        };
        // remainder
        let remainder: Vec<String> = args.collect();
        // configuration_directory
        let configuration_directory = match env::var("RUST_STOCK_TRACKER_CONFIGURATION_DIRECTORY") {
            Ok(x) if x != "" => PathBuf::from(x),
            _ => PathBuf::from( match dirs::home_dir() {
                Some(p) => p.join(".rust_stock_tracker"),
                None => return Err(ConfigHomeDirectoryNotFoundError),
            }),
        };

        // Checking validity
        //  remainder
        if (remainder.len() as i32) < command.num_args() { // Check if valid # of args have been provided
            return Err(ConfigArgumentsError(format!("{}",command)));
        }
        //  configuration_directory
        if !configuration_directory.exists() {
            let configuration_directory_c = configuration_directory.clone();
            fs::create_dir_all(&configuration_directory).map_err(|_| ConfigCreateDirectoryError(configuration_directory_c))?;
        }

        Ok(Config { command, remainder, configuration_directory})
    }

    pub fn hashmap_path(&self) -> PathBuf {
        self.configuration_directory.join("HashMap.JSON")
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
fn init(config: Config) -> Result<(), ProjectError> {
    let hashmap = HashMap::<String, User>::new();
    write_to_hashmap(&config.hashmap_path(), &hashmap)
}

/// The `create` function opens the HashMap and inserts a new user. 
fn create(config: Config) -> Result<(), ProjectError> {

    let username = &config.remainder[0];

    let f = |hashmap: &mut HashMap<String, User>| {
        hashmap.try_insert(String::from(username), User::new().map_err(|_| UserNewError)?)
        .map_or_else(|_| Err(HashMapInsertError(String::from(username))), |_| Ok(()))
    };

    modify_hashmap(&config.hashmap_path(), f)
}

/// The `delete` function queries the user for a confirmation, opens the HashMap, and deletes a user.
fn delete(config: Config) -> Result<(), ProjectError> {
    
    let username = &config.remainder[0];

    // Make sure the user wants to delete
    println!("Are you sure you want to delete user profile {}", username.to_string());

    let mut ans = String::new();
    io::stdin().read_line(&mut ans).map_err(|_| UserNewError)?;

    // Remove the newline
    let ans = ans.trim();

    // Debug
    //println!("fc: {}, lc: {}, lc == yes: {}", ans, ans.to_lowercase(), ans.to_lowercase().as_str() == "yes");

    match ans.to_lowercase().as_str() {
        // In the case where the user is sure
        "y" | "yes" => {
            let f = |hashmap: &mut HashMap<String, User>| hashmap
                .remove(&username.to_string()) // Remove
                .ok_or_else(|| HashMapRemoveError(username.to_string())).map(|_| ()); // Handle Option -> Result & discarding User
            modify_hashmap(&config.hashmap_path(), f)
        },
        // In the case where the user declines
        "q" | "quit" | "n" | "no" => Ok(()),
        // In the case where the user input is not recognized
        _ => Err(InvalidInputError),
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
fn read_from_hashmap<P: AsRef<Path>>(path: &P) -> Result<HashMap<String, User>, ProjectError> {

    let file = match fs::File::open(path) {
        Ok(x) => x,
        Err(_) => return Err(IOHashMapOpenError(PathBuf::from(path.as_ref())))
    };

    let reader = io::BufReader::new(&file);

    serde_json::from_reader(reader).map_err(|_| DeserializeJSONError(PathBuf::from(path.as_ref())))
}

/// The 'write_to_hashmap` function takes a `Path` and a `HashMap<String, User>` and writes the
/// `HashMap<String, User>` to the file located at that path using `serde_JSON` to write the file.
fn write_to_hashmap<P: AsRef<Path>>(path: &P, hashmap: &HashMap<String, User>) -> Result<(), ProjectError> {
    
    let serialized_hashmap = serde_json::to_string(hashmap).map_err(|_| SerializeJSONError)?;

    let mut file = match fs::File::create(path) {
        Ok(x) => x,
        Err(_) => return Err(IOHashMapOpenError(PathBuf::from(path.as_ref()))),
    };

    file.write_all(serialized_hashmap.as_bytes()).map_err(|_| IOHashMapWriteError(PathBuf::from(path.as_ref())))
}

fn modify_hashmap<P, F>(path: &P, f: F) -> Result<(), ProjectError> where 
    P: AsRef<Path>,
    F: Fn(&mut HashMap<String, User>) -> Result<(), ProjectError> {
    
    let hashmap = &mut read_from_hashmap(path)?;
    f(hashmap)?;
    write_to_hashmap(path, &hashmap)
}

// Testing

#[cfg(test)]
mod tests {
    use super::*;

    mod config_tests {

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
}