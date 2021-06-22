//! #stock_tracker
//!
//! This is the runtime logic for the rust_stock_tracker project

// features
#![feature(map_try_insert)]

mod user;
mod stock;

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


// external cratesAs `﻿stock.rs﻿` isn't meant to me a member of `﻿user.rs﻿`, tho
use dirs;
use serde::{Serialize, Deserialize}; // So we may prepare the HashMap to be written to a file
use serde_json; // So we may write and read the HashMap to JSON
use thiserror::Error; // For more structured definition of errors

// modules
use user::User;
use ProjectError::*; // To increse readability

/// The `ProjectError` enum represents the variants of `Error`s expected in `stock_tracker`
#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("Read from HashMap file {} unsuccessful", .0.display())]
    IOHashMapOpenError(PathBuf),
    #[error("Write to HashMap file at {} unsuccessful", .0.display())]
    IOHashMapWriteError(PathBuf),
    #[error("Read from State file {} unsuccessful", .0.display())]
    IOStateOpenError(PathBuf),
    #[error("Write to State file at {} unsuccessful", .0.display())]
    IOStateWriteError(PathBuf),
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
    #[error("Unexpected error: attempted to login as user {0}, but user {0} was not found.")]
    StateInvalidUserError(String),
    #[error("Input not recognized.")]
    InvalidInputError,
}

/// The 'UserCommand' enum represents the variety of input cases relating to users
#[derive(Debug)]
pub enum UserCommand {
    Create,
    Delete,
    Login,
    Logout,
    Showall,
}

/// The 'StockCommand' enum represents the variety of input cases relating to stocks
#[derive(Debug)]
pub enum StockCommand {
    Create,
}

/// The `Command` enum represents the variety of input cases a user could specify.
#[derive(Debug)]
pub enum Command {
    Init,
    // Zero State Commands
    UserC(UserCommand),
    // Logged In Commands
    StockC(StockCommand),
}


impl Command {

    /// Constructor for the `Command` enum to parse a `String` input
    pub fn new(s: &str) -> Result<Command, ProjectError> {
        Ok(match String::from(s).to_lowercase().as_str() {
            // Zero State Commands
            "i" | "init" => Command::Init,
            "cu" | "create-user" => Command::UserC(UserCommand::Create),
            "du" | "delete-user" => Command::UserC(UserCommand::Delete),
            "li" | "login" => Command::UserC(UserCommand::Login),
            "lo" | "logout" => Command::UserC(UserCommand::Logout),
            "sa" | "showall" => Command::UserC(UserCommand::Showall),
            // Logged In Commands
            "cs" | "create-stock" => Command::StockC(StockCommand::Create),
            _ => return Err(CommandInvalidError),
        })
    }

    /// Returns the number of arguments expected after the `Command`
    pub fn num_args(&self) -> i32 {
        match self {
            // Zero State Commands
            Command::Init => 0,
            Command::UserC(UserCommand::Create) => 1,
            Command::UserC(UserCommand::Delete) => 1,
            Command::UserC(UserCommand::Login) => 1,
            Command::UserC(UserCommand::Logout) => 0,
            Command::UserC(UserCommand::Showall) => 0,
            // Logged In Commands
            Command::StockC(StockCommand::Create)  => 1,
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self{
            // Zero State Commands
            Command::Init => "init",
            Command::UserC(UserCommand::Create) => "create-user",
            Command::UserC(UserCommand::Delete) => "delete-user",
            Command::UserC(UserCommand::Login) => "login",
            Command::UserC(UserCommand::Logout) => "logout",
            Command::UserC(UserCommand::Showall) => "showall",
            // Logged In Commands
            Command::StockC(StockCommand::Create) => "create-stock"
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

    /// Simple method to return the location of the UserMap 
    pub fn user_map_path(&self) -> PathBuf {
        self.configuration_directory.join("UserMap.JSON")
    }

    /// Simple mthod to return the location of the StockMap
    pub fn stock_map_path(&self) -> PathBuf {
        self.configuration_directory.join("StockMap.JSON")
    }
}

/// The `State` struct represents all persistency between calls to this program, such as logged-in states
#[derive(Serialize, Deserialize, Debug)]
pub struct State {
    /// A `bool` which is `true` if a user is logged in and `false` if no user is logged in.
    logged_in: bool,
    /// A `String` which, when `Some(x)`, `x` should always be a key of the HashMap in `UserMap.JSON`. When `logged_in` is
    /// `false`, `current_user` should be `None`.
    current_user: Option<String>,
}

impl State {

    /// `new()` is more flexible than `init()` and can be used to create a `State` from any existing file.
    pub fn new<P: AsRef<Path>>(path: &P) -> Result<State, ProjectError> {
        let file = match fs::File::open(path) {
            Ok(x) => x,
            Err(_) => return Err(IOStateOpenError(PathBuf::from(path.as_ref())))
        };

        let reader = io::BufReader::new(&file);

        serde_json::from_reader(reader).map_err(|_| DeserializeJSONError(PathBuf::from(path.as_ref())))
    }

    /// This function is like `new()`, but it checks if the path is initialized first and
    /// creates it if not. Whereas `new` expects a path to the file, `init()` only expects
    /// a `Config`.
    pub fn init(config: &Config) -> Result<State, ProjectError> {
        
        let path = &config.configuration_directory.join("State.JSON");

        if path.exists() {
            State::new(path)
        }
        else {
            let state = State { logged_in: false, current_user: None, };
            let serialized_state = serde_json::to_string(&state).map_err(|_| SerializeJSONError)?;

            let mut file = match fs::File::create(path) {
                Ok(x) => x,
                Err(_) => return Err(IOStateOpenError(PathBuf::from(path)))
            };

            file.write_all(serialized_state.as_bytes()).map_err(|_| IOStateWriteError(PathBuf::from(path)))?;

            Ok(state)
        }
    }

    /// `set_user()` simply sets the state to logged_in, applies the username provided to `current_user` and writes
    /// this to the state file.
    pub fn set_user(&mut self, config: Config, username: &str) -> Result<(), ProjectError> {
        self.logged_in = true;
        self.current_user = Some(String::from(username));
        self.write(config)
    }

    /// `try_set_user()` attempts to set the user to `username`, but checks the `HashMap` provided to ensure that it is
    /// valid before returning. Like `set_user()`, this method returns a result.
    pub fn try_set_user(&mut self, config: Config, username: &str, hashmap: HashMap<String, User>) -> Result<(), ProjectError> {
        if !self.valid_user(username, hashmap) {
            return Err(StateInvalidUserError(String::from(username)))
        } else {
            self.logged_in = true;
            self.current_user = Some(String::from(username));
            self.write(config)
        }
    }

    /// Returns to a "logged out" state
    pub fn clear_user(&mut self, config: Config) -> Result<(), ProjectError> {
        self.logged_in = false;
        self.current_user = None;
        self.write(config)
    }

    pub fn write(&self, config: Config) -> Result<(), ProjectError> {
        let path = &config.configuration_directory.join("State.JSON");

        let mut file = match fs::File::create(path) {
            Ok(x) => x,
            Err(_) => return Err(IOStateOpenError(PathBuf::from(path)))
        };

        let serialized_state = serde_json::to_string(self).map_err(|_| SerializeJSONError)?;

        file.write_all(serialized_state.as_bytes()).map_err(|_| IOStateWriteError(PathBuf::from(path)))?;

        Ok(())
    }

    /// Simple function that reports to the user if the `current_user` field is valid
    pub fn valid_state(&self, hashmap: HashMap<String, User>) -> bool {
        match &self.current_user {
            Some(x) => hashmap.contains_key(x),
            None => false,
        }
    }

    /// Simple function that reports to the user if the username provided is valid current_user
    pub fn valid_user(&self, username: &str, hashmap: HashMap<String, User>) -> bool {
        hashmap.contains_key(username)
    }
}

/// The `run` function represents the runtime logic of the program
pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    match config.command {
        Command::Init => init(config)?,
        Command::UserC(UserCommand::Create) => create_user(config)?,
        Command::UserC(UserCommand::Delete) => delete_user(config)?,
        Command::UserC(UserCommand::Login) => login(config)?,
        Command::UserC(UserCommand::Logout) => logout(config)?,
        Command::UserC(UserCommand::Showall) => showall(config)?,
        Command::StockC(StockCommand::Create) => create_stock(config)?,
    };

    Ok(())
}

//
// User-actuated functions
//

/// The `init` function produces a HashMap at a default location
fn init(config: Config) -> Result<(), ProjectError> {
    let hashmap = HashMap::<String, User>::new();
    write_to_hashmap(&config.user_map_path(), &hashmap)
}

/// The `create_user` function opens the HashMap and inserts a new user. 
fn create_user(config: Config) -> Result<(), ProjectError> {

    let username = &config.remainder[0];

    let f = |hashmap: &mut HashMap<String, User>| {
        hashmap.try_insert(String::from(username), User::new().map_err(|_| UserNewError)?)
        .map_or_else(|_| Err(HashMapInsertError(String::from(username))), |_| Ok(()))
    };

    modify_hashmap(&config.user_map_path(), f)
}

/// The `delete_user` function queries the user for a confirmation, opens the HashMap, and deletes a user.
fn delete_user(config: Config) -> Result<(), ProjectError> {
    
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
            modify_hashmap(&config.user_map_path(), f)
        },
        // In the case where the user declines
        "q" | "quit" | "n" | "no" => Ok(()),
        // In the case where the user input is not recognized
        _ => Err(InvalidInputError),
    }
}

/// The `login` function queries the user for a password, opens the HashMap, and activates a state where certain commmands will be applied on the user in question.
fn login(config: Config) -> Result<(), Box<dyn Error>>{
    // Setup
    let username = String::from(&config.remainder[0]);
    let mut state = State::init(&config)?;
    let hashmap = read_from_hashmap(&config.user_map_path())?;
    // Login
    state.try_set_user(config, &username, hashmap)?;
    println!("Logged in as {} successfully.", username);
    Ok(())
}

/// The `logout` function deactivates the state where certain commands will be applied on the user in question.
fn logout(config: Config) -> Result<(), ProjectError>{
    let mut state = State::init(&config)?;
    state.clear_user(config)?;
    println!("Logged out successfully.");
    Ok(())
}

/// The `showall` function relies on a logged in state and shows the current state of all the logged in user's stoc<P: AsRef<Path>>ks.
fn showall(config: Config) -> Result<(), ProjectError>{
    unimplemented!()
}

fn create_stock(config: Config) -> Result<(), ProjectError>{
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     mod config_tests {

//         use super::*;

//         #[test]
//         fn config_new_no_args() {
//             assert!(match Config::new(Vec::<String>::new().into_iter()) {
//                 Ok(_) => false,
//                 Err(x) => x == "Didn't get a command string",
//             });
//         }

//         #[test]
//         fn config_new_one_arg() {
//             assert!(match Config::new(vec![String::from("test1")].into_iter()) {
//                 Ok(_) => false,
//                 Err(x) => x == "Didn't get a command string",
//             });
//         }

//         #[test]
//         fn config_new_two_invalid_args() {
//             assert!(
//                 match Config::new(vec![String::from("test1"), String::from("test2")].into_iter()) {
//                     Ok(_) => false,
//                     Err(x) => x == "Invalid command string",
//                 }
//             );
//         }

//         #[test]
//         fn config_new_many_invalid_args() {
//             let mut check = true;

//             for i in 3..100 {
//                 let mut v = Vec::<String>::new();
//                 for j in 0..i {
//                     v.push(format!("test{}", j+1));
//                 }
//                 check = check &&
//                     match Config::new(v.clone().into_iter()) {
//                         Ok(_) => false,
//                         Err(x) => x == "Invalid command string",
//                     }
//             }

//             assert!(check);
//         }

//         #[test]
//         fn config_new_two_valid_args() {
//             assert!(
//                 match Config::new(vec![String::from("test1"), String::from("showall")].into_iter()) {
//                     Ok(Config {command: Command::Showall, ..}) => true,
//                     _ => false,
//                 }
//             );
//         }

//         #[test]
//         fn config_new_two_valid_args_invalid_num_args() {
//             assert!(
//                 match Config::new(vec![String::from("test1"), String::from("create")].into_iter()) {
//                     Ok(_) => false,
//                     Err(x) => x == "Too few arguments provided for create",
//                 }
//             );
//         }    

//         #[test]
//         fn config_new_three_valid_args() {
//             assert!(
//                 match Config::new(vec![String::from("test1"), String::from("create"), String::from("test3")].into_iter()) {
//                     Ok(Config {
//                         command: Command::Create,
//                         remainder,
//                     }) => remainder == vec![String::from("test3")],
//                     _ => false,
//                 }
//             );
//         }
//     }
// }