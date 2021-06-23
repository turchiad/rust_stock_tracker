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

/// A complete representation of a user and all of their corresponding data.
#[derive(Serialize, Deserialize, Clone, Debug, Display)]
#[display(fmt = "{} {}", first_name, last_name)]
pub struct User<'a> {
    /// A user's username. Special characters such as !,?,&,| are not valid.
    username: String,

    /// A user's first name
    first_name: String,
    /// A user's last name
    last_name: String,
    /// A user's middle initial
    middle_initial: String,
    /// A collection of the user's stocks
    pub portfolio: Option::<HashMap::<String, StockUnit<'a>>>,
}


impl User<'_> {
    pub fn new() -> Result<User<'static>, ProjectError> {
        return Ok(User {
            username: String::from("username"),
            first_name: String::from("first_name"),
            last_name: String::from("last_name"),
            middle_initial: String::from("middle_initial"),
            portfolio: None,
        })
    }

    pub fn add_stock(&mut self, stock: &Stock, qt: u32) -> Result<(), ProjectError> {
        match self.portfolio {
            Some(hashmap) => match hashmap.try_insert(stock.ticker.clone(), StockUnit::new(&stock, qt)?) {
                    Ok(_) => Ok(()),
                    Err(_) => self.add_stock_additional(stock, qt),
                }
        }
    }

    fn add_stock_additional(&self, stock: &Stock, qt: u32) -> Result<(), ProjectError> {
        unimplemented!()
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
