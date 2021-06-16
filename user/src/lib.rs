//! #user
//!
//! This holds the `User` type and related methods

// std
use std::collections::HashMap;

// external crates
use serde::{Serialize, Deserialize}; // So we may prepare the HashMap to be written to a file
use derive_more::{Display}; // So we may derive Display

// internal crates
use stock;

/// A complete representation of a user and all of their corresponding data.
#[derive(Serialize, Deserialize, Debug, Display)]
#[display(fmt = "{} {}", first_name, last_name)]
pub struct User {
    /// A user's username. Special characters such as !,?,&,| are not valid.
    username: String,
    /// A user's first name
    first_name: String,
    /// A user's last name
    last_name: String,
    /// A user's middle initial
    middle_initial: String,
    /// A collection of the user's stocks
    portfolio: HashMap<String, stock::Stock>,
}


impl User {
    pub fn new() -> Result<User, String> {
        return Ok(User {
            username: String::from("username"),
            first_name: String::from("first_name"),
            last_name: String::from("last_name"),
            middle_initial: String::from("middle_initial"),
            portfolio: HashMap::<String, stock::Stock>::new(),
        })
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
