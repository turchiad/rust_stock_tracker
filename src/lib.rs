//! #stock_tracker
//!
//! This is the runtime logic for the rust_stock_tracker project

// features
#![feature(map_try_insert)]

// modules
mod command;
mod error;
mod stock;
mod user;

use crate::command::*;
use crate::error::ProjectError;
use crate::error::ProjectError::*;
use crate::stock::Stock;
use crate::user::User;

// std
use std::collections::HashMap; // So we may construct HashMaps
use std::collections::BTreeMap; // So we may construct BTreeMaps
use std::env; // So we can set the configuration path by environment variables
use std::fs; // So we may read/write to files.
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

// external crates
use dirs;
use serde::{Serialize, Deserialize}; // So we may prepare the HashMap to be written to a file
use serde_json; // So we may write and read the HashMap to JSON

/// The `Config` struct represents the CLI input state of a call to this program.
#[derive(Debug, Clone)]
pub struct Config {
    /// The primary command immediately following the call
    pub command: Command,
    /// The remainder of arguments which may be processed differently depending on the command.
    pub remainder: Vec<String>,
    /// The location of the program's configuration files
    pub configuration_directory: PathBuf, 
}

impl Config {
    pub fn new<I, T> (args: I) -> Result<Config, ProjectError> where
    I: Iterator<Item = T>,
    T: Into<String>, {
        // Convert to `T` to `String`
        let mut args = args.map(Into::into);

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
    pub fn set_user(&mut self, config: &Config, username: &str) -> Result<(), ProjectError> {
        self.logged_in = true;
        self.current_user = Some(String::from(username));
        self.write(config)
    }

    /// `try_set_user()` attempts to set the user to `username`, but checks the `HashMap` provided to ensure that it is
    /// valid before returning. Like `set_user()`, this method returns a result.
    pub fn try_set_user(&mut self, config: &Config, username: &str, hashmap: HashMap<String, User>) -> Result<(), ProjectError> {
        if !self.valid_user(username, hashmap) {
            return Err(StateInvalidUserError(String::from(username)))
        } else {
            self.logged_in = true;
            self.current_user = Some(String::from(username));
            self.write(config)
        }
    }

    /// Returns to a "logged out" state
    pub fn clear_user(&mut self, config: &Config) -> Result<(), ProjectError> {
        self.logged_in = false;
        self.current_user = None;
        self.write(config)
    }

    pub fn write(&self, config: &Config) -> Result<(), ProjectError> {
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
pub fn run(config: &Config) -> Result<(), ProjectError> {
    match config.command {
        // Special Commands
        Command::Init => init(config)?,
        Command::Console => console_mode(config)?,
        Command::Exit => return Err(InvalidInputError), // should only be accessible from within console_mode
        // State Commands
        Command::StateC(StateCommand::Login)            => login(config)?,
        Command::StateC(StateCommand::Logout)           => logout(config)?,
        // User Commands
        Command::UserC(UserCommand::Create)             => create_user(config)?,
        Command::UserC(UserCommand::Delete)             => delete_user(config)?,
        Command::UserC(UserCommand::Edit)               => edit_user(config)?,
        Command::UserC(UserCommand::List)               => list_users(config)?,
        // Stock Commands
        Command::StockC(StockCommand::Create)           => create_stock(config)?,
        Command::StockC(StockCommand::Delete)           => delete_stock(config)?,
        Command::StockC(StockCommand::Edit)             => edit_stock(config)?,
        Command::StockC(StockCommand::List)             => list_stocks(config)?,
        // Portfolio Command
        Command::PortfolioC(PortfolioCommand::Buy)      => buy_stock(config)?,
        Command::PortfolioC(PortfolioCommand::List)     => list_portfolio(config)?,
    };

    Ok(())
}

//
// User-actuated functions
//

/// The `init` function produces a HashMap at a default location
fn init(config: &Config) -> Result<(), ProjectError> {
    // Generate new user hashmap and write to file
    let user_map = HashMap::<String, User>::new();
    write_to_hashmap(&config.user_map_path(), &user_map)?;
    // Generate new stock hashmap and write to file
    let stock_map = HashMap::<String, Stock>::new();
    write_to_hashmap(&config.stock_map_path(), &stock_map)?;
    // Log any users out of state so there are no impossible users logged in
    let mut state = State::init(config)?;
    state.clear_user(config)?;

    notify("All user/stock data reset/initialized.");
    Ok(())
}

fn console_mode(_config: &Config) -> Result<(), ProjectError> {
    // Notify the user that they have entered console mode
    notify("Entering console mode...");
    
    loop { // Loop until exited
        print!(">");
        io::stdout().flush().unwrap();

        let mut args_string = String::new();
        io::stdin()
            .read_line(&mut args_string)
            .map_err(|_| InvalidInputError)?;

        // The config constructor expects the first argument to be the call to the program
        let args = format!("filler {}", args_string); // To satisfy macro requirements this is on its own line
        let args = args.split_ascii_whitespace();
        // Construct a new config for the console-mode loop instance
        let this_config = match Config::new(args) {
            Ok(x) => x,
            Err(_) => { notify("Command not recognized."); continue},
        };

        // Accept command inputs
        let result = match this_config.command {
            // Special Commands
            Command::Init                                   => init(&this_config),
            Command::Console                                => { notify("Already in console mode."); continue },
            Command::Exit                                   => { notify("Exiting..."); return Ok(()) }, // should only be accessible from within console_mode
            // State Commands
            Command::StateC(StateCommand::Login)            => login(&this_config),
            Command::StateC(StateCommand::Logout)           => logout(&this_config),
            // User Commands
            Command::UserC(UserCommand::Create)             => create_user(&this_config),
            Command::UserC(UserCommand::Delete)             => delete_user(&this_config),
            Command::UserC(UserCommand::Edit)               => edit_user(&this_config),
            Command::UserC(UserCommand::List)               => list_users(&this_config),
            // State Commands
            Command::StockC(StockCommand::Create)           => create_stock(&this_config),
            Command::StockC(StockCommand::Delete)           => delete_stock(&this_config),
            Command::StockC(StockCommand::Edit)             => edit_stock(&this_config),
            Command::StockC(StockCommand::List)             => list_stocks(&this_config),
            // Portfolio Commands
            Command::PortfolioC(PortfolioCommand::Buy)      => buy_stock(&this_config),
            Command::PortfolioC(PortfolioCommand::List)     => list_portfolio(&this_config),
        };
        // Check if Error command should throw exit console mode or not
        match result {
            Ok(_) => continue,
            Err(x @ InputParseError(_,_)) |
            Err(x @ HashMapKeyNotFoundError(_)) |
            Err(x @ StateNoUserError) | 
            Err(x @ StateInvalidUserError(_)) => println!("{}", x),
            Err(x) => return Err(x),
        };
    }
}

/// The `login` function opens the HashMap, and activates a state where certain commmands will be applied on the user in question.
fn login(config: &Config) -> Result<(), ProjectError>{
    // Setup
    let username = &config.remainder[0];
    let mut state = State::init(config)?;
    let hashmap = read_from_hashmap(&config.user_map_path())?;
    // Login
    state.try_set_user(config, &username, hashmap)?;
    notify(&format!("Logged in as {} successfully.", username));
    Ok(())
}

/// The `logout` function deactivates the state where certain commands will be applied on the user in question.
fn logout(config: &Config) -> Result<(), ProjectError>{
    let mut state = State::init(config)?;
    state.clear_user(config)?;
    notify("Logged out successfully.");
    Ok(())
}

/// The `create_user` function opens the HashMap and inserts a new user. 
fn create_user(config: &Config) -> Result<(), ProjectError> {

    let username = &config.remainder[0];

    let f = |hashmap: &mut HashMap<String, User>| {
        hashmap.try_insert(String::from(username), User::new_from_username(username).map_err(|_| UserNewError)?)
        .map_or_else(|_| Err(HashMapInsertError(String::from(username))), |_| Ok(()))
    };

    modify_hashmap(&config.user_map_path(), f)?;

    notify(&format!("User {} has been added.", username));
    Ok(())
}

/// The `delete_user` function queries the user for a confirmation, opens the HashMap, and deletes a user.
fn delete_user(config: &Config) -> Result<(), ProjectError> {
    
    let username = &config.remainder[0];

    // Preliminary check if username exists in the user map
    if !read_from_hashmap::<PathBuf, User>(&config.user_map_path())?.contains_key(username) {
        return Err(HashMapKeyNotFoundError(String::from(username)))
    }

    // Make sure the user wants to delete
    println!("Are you sure you want to delete user profile {}", username.to_string());

    let mut ans = String::new();
    io::stdin().read_line(&mut ans).map_err(|_| UserNewError)?;

    // Remove the newline
    let ans = ans.trim();

    match ans.to_lowercase().as_str() {
        // In the case where the user is sure
        "y" | "yes" => {
            let f = |hashmap: &mut HashMap<String, User>| hashmap
                .remove(&username.to_string()) // Remove
                .ok_or_else(|| HashMapRemoveError(String::from(username))).map(|_| ()); // Handle Option -> Result & discarding User
            modify_hashmap(&config.user_map_path(), f)?
        },
        // In the case where the user declines
        "q" | "quit" | "n" | "no" => return Ok(()),
        // In the case where the user input is not recognized
        _ => return Err(InvalidInputError),
    };

    notify(&format!("User {} deleted.", username));
    Ok(())
}

/// The `edit_user` function takes a user id, a property, and some value, and allows the user to modify the property of the 
/// `User` matching the user id to the specified value, before saving the `User`.
fn edit_user(config: &Config) -> Result<(), ProjectError> {
    // Reading user input 1
    let username = &config.remainder[0];

    let mut user_map: HashMap<String, User> = read_from_hashmap(&config.user_map_path())?;
    let user = if !user_map.contains_key(username) {
        return Err(HashMapKeyNotFoundError(String::from(username)))
    } else {
        user_map.get_mut(username).unwrap() // We can be confident this will be Some()
    };

    // Reading user input 2
    let property = String::from(&config.remainder[1]);
    let value = String::from(&config.remainder[2]);

    // Do we need to update the key or `State`?
    let mut update_username = false;
    // New username placeholders for the `HashMap` key and the `State` current user (if needed)
    let mut new_username_1 = String::from("");
    let mut new_username_2 = String::from("");

    // Notify message (we want this to be displayed after the successful write)
    let note;

    match user.get_property(&property)? {
        user::Property::Username(x) => { // Must be a `String`
            let username_property = x;
            *username_property = value.clone(); // We clone here so we don't move `value`
            // We will need to update the username as a key
            update_username = true;
            // We will need two owned clones of the username to insert as a key and (possibly) to insert into the `State`
            new_username_1 = value.clone();
            new_username_2 = value;
            // Note
            note = format!("User {} changed to {}.", username, username_property);
        },
        user::Property::FirstName(x) => { // Must be a `String`
            let first_name = x;
            *first_name = value;
            note = format!("User {}'s first name changed to {}.", username, first_name);
        },
        user::Property::LastName(x) => { // Must be a `String`
            let last_name = x;
            *last_name = value;
            note = format!("User {}'s last name changed to {}.", username, last_name);
        },
        user::Property::MiddleInitial(x) => { // Must be a `String`
            let middle_initial = x;
            *middle_initial = value;
            note = format!("User {}'s middle initial changed to {}.", username, middle_initial);
        },
    };

    // Remove old entry from HashMap if necessary
    if update_username {
        let user = user_map.remove(username).unwrap(); // We can be confident this is `Some`
        user_map.insert(new_username_1, user);
    }

    // Write to hashmap
    write_to_hashmap(&config.user_map_path(), &user_map)?;

    // Update state if necessary
    let mut state = State::init(config)?;
    // If the user we changed is the one logged in
    if update_username &&
        match &state.current_user { Some(x) => *x == new_username_2, None => false, } {
        state.set_user(config, &new_username_2)?;
    }

    // Notify success
    notify(&note);

    Ok(())
}

/// The `list_users` function lists all created `User`s in the `UserMap`
fn list_users(config: &Config) -> Result<(), ProjectError> {
    // Read user_map
    let user_map: HashMap<String, User> = read_from_hashmap(&config.user_map_path())?;

    // If stock_map is empty, tell the user and end short
    if user_map.is_empty() {
        println!("No users created.");
        return Ok(())
    }

    // Sort the HashMap by key
    let list: BTreeMap<String, User> = user_map.into_iter().collect();

    println!("List of users:");
    for (_, user) in list {
        println!("{}", user);
    }

    Ok(())
}


/// The `create_stock` function opens the StockMap and inserts a new stock.
fn create_stock(config: &Config) -> Result<(), ProjectError> {
    let stock_id = &config.remainder[0];

    let f = |hashmap: &mut HashMap<String, Stock>| {
        hashmap.try_insert(String::from(stock_id), Stock::new_from_ticker(stock_id).map_err(|_| StockNewError)?)
        .map_or_else(|_| Err(HashMapInsertError(String::from(stock_id))), |_| Ok(()))
    };

    modify_hashmap(&config.stock_map_path(), f)?;
    notify(&format!("Stock {} has been added.", stock_id));
    Ok(())
}

/// The `delete_stock` function queries the user for a confirmation, opens the StockMap, and deletes a Stock.
fn delete_stock(config: &Config) -> Result<(), ProjectError>{
    // Read stock data
    let stock_id = &config.remainder[0];

    // Preliminary check if stock exists in the user map
    if !read_from_hashmap::<PathBuf, Stock>(&config.stock_map_path())?.contains_key(stock_id) {
        return Err(HashMapKeyNotFoundError(String::from(stock_id)))
    }


    // Make sure the user wants to delete
    println!("Are you sure you want to delete stock {}", stock_id.to_string());
    let mut ans = String::new();
    io::stdin().read_line(&mut ans).map_err(|_| UserNewError)?;
    let ans = ans.trim(); // Remove the newline
    match ans.to_lowercase().as_str() {
        // In the case where the user is sure
        "y" | "yes" => {
            let f = |hashmap: &mut HashMap<String, Stock>| hashmap
                .remove(&stock_id.to_string()) // Remove
                .ok_or_else(|| HashMapRemoveError(stock_id.to_string())).map(|_| ()); // Handle Option -> Result & discarding User
            modify_hashmap(&config.stock_map_path(), f)?;
        },
        // In the case where the user declines
        "q" | "quit" | "n" | "no" => return Ok(()),
        // In the case where the user input is not recognized
        _ => return Err(InvalidInputError),
    }

    // Closeout
    notify(&format!("Stock {} has been deleted.", stock_id));
    Ok(())
}

/// The `edit_stock` function takes a stock ticker id, a property, and some value, and allows the user to modify the property of the 
/// `Stock` matching the stock ticker id to the specified value, before saving the stock. Note: this does not change a user's `Stock`s to
/// the updated version, which will have the be done on the user-side.
fn edit_stock(config: &Config) -> Result<(), ProjectError> {
    // Reading user input 1
    let stock_id = &config.remainder[0];

    let mut stock_map: HashMap<String, Stock> = read_from_hashmap(&config.stock_map_path())?;
    let stock = if !stock_map.contains_key(stock_id) {
        return Err(HashMapKeyNotFoundError(String::from(stock_id)))
    } else {
        stock_map.get_mut(stock_id).unwrap() // We can be confident this will be Some()
    };

    // Reading user input 2
    let property = String::from(&config.remainder[1]);
    let value = String::from(&config.remainder[2]);

    // Do we need to update the key?
    let mut update_stock_id = false;
    // New username placeholders for the `HashMap` key and the `State` current user (if needed)
    let mut new_stock_id = String::from("");

    // Notify message (we want this to be displayed after the successful write)
    let note;

    match stock.get_property(&property)? {
        stock::Property::Ticker(x) => { // Must be a `String`
            let ticker = x;
            *ticker = value.clone();
            // We will need to update the stock_id as a key
            update_stock_id = true;
            new_stock_id = value.clone();
            // Note
            note = format!("Stock {} changed to {}.", stock_id, ticker);
        },
        stock::Property::CompanyName(x) => { // Must be a `String`
            let company_name = x;
            *company_name = value;
            note = format!("Stock {} changed to {}.", stock_id, company_name);
        },
        stock::Property::Value(x) => { // Must be a `f64`
            let stock_value = x;
            let value = parse_or_err::<f64>(&value)?; // Convert `value` to `f64`
            *stock_value = value;
            note = format!("Stock {} changed to {}.", stock_id, stock_value);
        },
    };

    // Remove old entry from HashMap if necessary
    if update_stock_id {
        let stock = stock_map.remove(stock_id).unwrap(); // We can be confident this is `Some`
        stock_map.insert(new_stock_id, stock);
    }

    // Write to hashmap
    write_to_hashmap(&config.stock_map_path(), &stock_map)?;

    // Notify success
    notify(&note);

    Ok(())
}

/// The `list_stocks` function lists all created `Stock`s in the `StockMap`
fn list_stocks(config: &Config) -> Result<(), ProjectError> {
    // Read stock_map
    let stock_map: HashMap<String, Stock> = read_from_hashmap(&config.stock_map_path())?;

    // If stock_map is empty, tell the user and end short
    if stock_map.is_empty() {
        println!("No stocks created.");
        return Ok(())
    }

    // Sort the HashMap by key
    let list: BTreeMap<String, Stock> = stock_map.into_iter().collect();

    println!("List of stocks:");
    for (_, stock) in list {
        println!("{}", stock);
    }

    Ok(())
}

/// The `buy_stock` function takes a stock ticker id and a quantity (in that order) and adds the quantity of purchased stocks
/// to the current user's `portfolio`, finishing by saving the user.
fn buy_stock(config: &Config) -> Result<(), ProjectError>{
    
    // Check user is logged in first
    let username = match State::init(&config)?.current_user {
        Some(x) => x,
        None => return Err(StateNoUserError),
    };

    // Read necessary stock data
    let stock_id = &config.remainder[0];
    let stock_qt = parse_or_err::<u32>(&config.remainder[1])?;
    
    // Check availability of stock and retrieve it if available
    let stock_map: HashMap<String, Stock> = read_from_hashmap(&config.stock_map_path())?;
    let stock = if !stock_map.contains_key(stock_id) {
        return Err(HashMapKeyNotFoundError(String::from(stock_id)))
    } else {
        stock_map.get(stock_id).unwrap() // We can be confident this will be Some()
    };

    // Check availability of user and retrieve it if available
    let mut user_map: HashMap<String, User> = read_from_hashmap(&config.user_map_path())?;
    let user = if !user_map.contains_key(&username) {
        return Err(HashMapKeyNotFoundError(String::from(username)))
    } else {
        user_map.get_mut(&username).unwrap() // We can be confident this will be Some()
    };

    // Alter user and write map.
    user.add_stock(stock, stock_qt)?;
    write_to_hashmap(&config.user_map_path(), &user_map)?;

    // Closeout
    notify(&format!("{} shares of stock {} purchased by {}", stock_qt, stock_id, username));
    Ok(())
}

/// The `list_portfolio` function relies on a logged in state and shows the current state of all the logged in user's stocks
fn list_portfolio(config: &Config) -> Result<(), ProjectError>{
    let username = match State::init(config)?.current_user {
        Some(x) => x,
        None => return Err(StateNoUserError),
    };

    let user_map: HashMap<String, User> = read_from_hashmap(&config.user_map_path())?;
    let user = if !user_map.contains_key(&username) {
        return Err(HashMapKeyNotFoundError(String::from(username)))
    } else {
        user_map.get(&username).unwrap() // We can be confident this will be Some()
    };

    println!("User profile {} has:", username);

    for (_, stock_unit) in match &user.portfolio {
        Some(x) => x.iter(),
        None => {
            println!("No holdings");
            return Ok(())
        },
    } {
        println!("{}: {} shares", stock_unit.stock.ticker, stock_unit.quantity);
    }

    Ok(())
}

//
// Assistive functions
//

/// The `notify` function is a simple function that prints the `&str` `s` to the screen. The puropose of this
/// function is to centralize functions that need to print a small notification message to the screen, such
/// that if the procedure of this behavior is to be changed in the future - it can be modified in one place.
fn notify(s: &str) {
    println!("{}",s);
}

/// The `parse_or_err<T>()` function is a simple wrapper function that will map the error output to a `ProjectError`
/// of the right type.
fn parse_or_err<T>(s: &String) -> Result<T, ProjectError> where T: std::str::FromStr {
    s.parse().map_err(|_| InputParseError(String::from(s), format!("{}", std::any::type_name::<T>())))
}

/// The `read_from_hashmap` function takes a `Path` and returns the `HashMap<String, T>` located at that path
/// using `serde_JSON` to read the file.
fn read_from_hashmap<P, T>(path: &P) -> Result<HashMap<String, T>, ProjectError> where
    P: AsRef<Path>,
    T: serde::de::DeserializeOwned, {
    let file = match fs::File::open(path) {
        Ok(x) => x,
        Err(_) => return Err(IOHashMapOpenError(PathBuf::from(path.as_ref())))
    };

    let reader = io::BufReader::new(&file);

    serde_json::from_reader(reader).map_err(|_| DeserializeJSONError(PathBuf::from(path.as_ref())))
}

/// The 'write_to_hashmap` function takes a `Path` and a `HashMap<String, User>` and writes the
/// `HashMap<String, User>` to the file located at that path using `serde_JSON` to write the file.
fn write_to_hashmap<P, T>(path: &P, hashmap: &HashMap<String, T>) -> Result<(), ProjectError> where
    P: AsRef<Path>,
    T: serde::ser::Serialize, {
    
    let serialized_hashmap = serde_json::to_string(hashmap).map_err(|_| SerializeJSONError)?;

    let mut file = match fs::File::create(path) {
        Ok(x) => x,
        Err(_) => return Err(IOHashMapOpenError(PathBuf::from(path.as_ref()))),
    };

    file.write_all(serialized_hashmap.as_bytes()).map_err(|_| IOHashMapWriteError(PathBuf::from(path.as_ref())))
}

fn modify_hashmap<P, F, T>(path: &P, f: F) -> Result<(), ProjectError> where 
    P: AsRef<Path>,
    F: Fn(&mut HashMap<String, T>) -> Result<(), ProjectError>,
    T: serde::ser::Serialize + serde::de::DeserializeOwned, {
    
    let hashmap = &mut read_from_hashmap(path)?;
    f(hashmap)?;
    write_to_hashmap::<P, T>(path, hashmap)
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