//! #stock
//!
//! This holds the `Stock` type and related methods

// external crates
use serde::{Serialize, Deserialize}; // So we may prepare the HashMap to be written to a file

// internal crates
use crate::error::ProjectError;
use crate::error::ProjectError::*;

/// A representative value of one share of a company's stock
#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Stock {
    /// A company's ticker, typically a series of capital letters e.g. FOO, BAR, etc.
    pub ticker: String,
    /// A company's name
    pub company_name: String,
    /// The USD value of one share of the company's stock.
    pub value: f64,
}

impl Stock {
    pub fn new() -> Result<Stock, ProjectError> {
        return Ok( Stock {
            ticker: String::from("ticker"),
            company_name: String::from("company_name"),
            value: 0.0,
        })
    }
}

/// A representative of amount of stocks one owns
#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct StockUnit<'a> {
    /// The Stock signature of the company
    pub stock: &'a Stock,
    /// The quantity of shares of `Stock`
    pub quantity: u32,
}

impl<'a> StockUnit<'a> {
    pub fn new(stock: &'a Stock, quantity: u32) -> Result<StockUnit, ProjectError> {
        return Ok( StockUnit {
            stock: stock,
            quantity: quantity,
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
