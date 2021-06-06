//! #stock_tracker
//!
//! This is the runtime logic for the rust_stock_tracker project

use std::error::Error; // So we may define Box<dyn Error> // To allow for the use of `env::Args` in setting up `Config`

pub struct Config {
    // The primary command immediately following the call
    pub command: String,
    // The remainder of arguments which may be processed differently depending on the command.
    pub remainder: Vec<String>,
}

impl Config {
    pub fn new<Args: Iterator<Item = String>>(mut args: Args) -> Result<Config, &'static str> {
        args.next(); // Discard the first argument

        let command = match args.next() {
            Some(arg) => arg,
            None => return Err("Didn't get a command string"),
        };

        let remainder = args.collect();

        Ok(Config { command, remainder })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_new_no_args() {
        assert!(match Config::new(Vec::<String>::new().into_iter()) {
            Ok(_) => false,
            Err(_) => true,
        });
    }

    #[test]
    fn config_new_one_arg() {
        assert!(match Config::new(vec![String::from("test1")].into_iter()) {
            Ok(_) => false,
            Err(_) => true,
        });
    }

    #[test]
    fn config_new_two_args() {
        assert!(
            match Config::new(vec![String::from("test1"), String::from("test2")].into_iter()) {
                Ok(_) => true,
                Err(_) => false,
            }
        );
    }

    #[test]
    fn config_new_many_args() {
        let mut check = true;

        for i in 3..100 {
            let mut v = Vec::<String>::new();
            for j in 0..i {
                v.push(format!("test{}", j));
            }
            check &=
                match Config::new(vec![String::from("test1"), String::from("test2")].into_iter()) {
                    Ok(_) => true,
                    Err(_) => false,
                }
        }

        assert!(check);
    }
}
