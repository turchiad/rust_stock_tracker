//! #user
//!
//! This holds the `User` type and related methods

use std::collections::HashMap;
use stock;

/// A complete representation of a user and all of their corresponding data.
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
