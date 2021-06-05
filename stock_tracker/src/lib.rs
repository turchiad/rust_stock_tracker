//! #stock_tracker
//!
//! This is the runtime logic for the rust_stock_tracker project

use std::env; // To allow for the use of `env::Args` in setting up `Config`

pub struct Config {
    // The primary command immediately following the call
    pub command: String,
    // The remainder of arguments which may be processed differently depending on the command.
    pub remainder: Vec<String>,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next(); // Discard the first argument

        let command = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a command string"),
        };

        let remainder = args.collect();

        Ok(Config { command, remainder })
    }
}

pub fn run() {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
