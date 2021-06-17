//! #stock
//!
//! This holds the `Stock` type and related methods

// std


// external crates
use serde::{Serialize, Deserialize}; // So we may prepare the HashMap to be written to a file

/// A representative value of one share of a company's stock
#[derive(Serialize, Clone, Deserialize, Debug)]
pub struct Stock {
    /// A company's ticker, typically a series of capital letters e.g. FOO, BAR, etc.
    ticker: String,
    /// A company's name
    company_name: String,
    /// The USD value of one share of the company's stock.
    value: f64,
}

impl Stock {
    pub fn new() -> Result<Stock, String> {
        return Ok( Stock {
            ticker: String::from("ticker"),
            company_name: String::from("company_name"),
            value: 0.0,
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
