//! #user
//!
//! This holds the `User` type and related methods

// std
use std::collections::HashMap;

// external crates
use serde::{Serialize, Deserialize}; // So we may prepare the HashMap to be written to a file
use derive_more::{Display}; // So we may derive Display

// internal crates
use crate::stock::Stock;
use crate::stock::StockUnit;
use crate::error::ProjectError;
use crate::error::ProjectError::*;

/// This `enum` exists to express the properties a user a might encounter in the `User.get_property()` method
#[derive(Debug)]
pub enum Property<'a> {
    Username(&'a mut String),
    FirstName(&'a mut String),
    LastName(&'a mut String),
    MiddleInitial(&'a mut String),
}

/// A complete representation of a user and all of their corresponding data.
#[derive(Serialize, Deserialize, Clone, Debug, Display)]
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
    pub portfolio: Option::<HashMap::<String, StockUnit>>,
}


impl User {

    pub fn new() -> Result<User, ProjectError> {
        return Ok(User {
            username: String::from("username"),
            first_name: String::from("first_name"),
            last_name: String::from("last_name"),
            middle_initial: String::from("middle_initial"),
            portfolio: None,
        })
    }

    /// The `get_property()` function returns a mutable reference to the property of the `User` requested based on a `String s`
    /// which matches the name of a `User`'s corresponding property
    pub fn get_property(&mut self, s: &str) -> Result<Property, ProjectError>{
        match String::from(s).to_lowercase().as_str() {
            "u" | "username"                            => Ok(Property::Username(&mut self.username)),
            "fn" | "first-name" | "firstname"           => Ok(Property::FirstName(&mut self.first_name)),
            "ln" | "last-name" | "lastname"             => Ok(Property::LastName(&mut self.last_name)),
            "mi" | "middle-initial" | "middleinitial"   => Ok(Property::MiddleInitial(&mut self.middle_initial)),
            _                                           => Err(InvalidInputError),
        }
    }

    /// The `add_stock()` function allows a user to add a `StockUnit` with a given `Stock` and `u32` quantity
    pub fn add_stock(&mut self, stock: &Stock, qt: u32) -> Result<(), ProjectError> {
        match &mut self.portfolio {
            Some(hashmap) => match hashmap.try_insert(stock.ticker.clone(), StockUnit::new(stock.clone(), qt)?) {
                    Ok(_) => {Ok(())},
                    Err(_) => self.add_stock_additional(stock, qt),
                }
            None => { // Generate a new hashmap for `portfolio` and add our stock_unit to it.
                let mut hashmap = HashMap::<String, StockUnit>::new();
                hashmap.insert(stock.ticker.clone(), StockUnit::new(stock.clone(), qt)?);
                self.portfolio = Some(hashmap);
                Ok(())
            },
        }
    }

    fn add_stock_additional(&mut self, stock: &Stock, qt: u32) -> Result<(), ProjectError> {
        match &mut self.portfolio {
            Some(hashmap) => {
                let stock_unit = hashmap.get_mut(&stock.ticker).unwrap(); // We can be confident get will be Some()
                stock_unit.add_stock(qt)
            }, None => Err(ImpossibleStateError)
        }
    }

}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
