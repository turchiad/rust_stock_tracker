//! #stock
//!
//! This holds the `stock` type and related methods

/// A representative value of one share of a company's stock
struct Stock {
    /// A company's ticker, typically a series of capital letters e.g. FOO, BAR, etc.
    ticker: String,
    /// A company's name
    company_name: String,
    /// The USD value of one share of the company's stock.
    value: f64,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
