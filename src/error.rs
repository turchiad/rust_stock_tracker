//! #user
//!
//! This holds the `Error` type and related methods

use std::path::PathBuf;
use thiserror::Error; // For more structured definition of errors

/// The `ProjectError` enum represents the variants of `Error`s expected in `stock_tracker`

#[derive(Error, Debug)]
pub enum ProjectError {
    #[error("Read from HashMap file {} unsuccessful.", .0.display())]
    IOHashMapOpenError(PathBuf),
    #[error("Write to HashMap file at {} unsuccessful.", .0.display())]
    IOHashMapWriteError(PathBuf),
    #[error("Read from State file {} unsuccessful.", .0.display())]
    IOStateOpenError(PathBuf),
    #[error("Write to State file at {} unsuccessful.", .0.display())]
    IOStateWriteError(PathBuf),
    #[error("Serialization unsuccessful.")]
    SerializeJSONError,
    #[error("Deserialization of JSON file {} unsuccessful.", .0.display())]
    DeserializeJSONError(PathBuf),
    #[error("Insertion to HashMap failed: key {0} is already occupied.")]
    HashMapInsertError(String),
    #[error("Remove from HashMap at key {0} unsuccessful.")]
    HashMapRemoveError(String),
    #[error("Key {0} not found in HashMap.")]
    HashMapKeyNotFoundError(String),
    #[error("Error creating new User.")]
    UserNewError,
    #[error("Error creating new Stock.")]
    StockNewError,
    #[error("Error parsing inputs, check that this call was formatted correctly.")]
    ParseError,
    #[error("No command string provided.")]
    ConfigNoCommandError,
    #[error("Too few arguments provided for {0}")]
    ConfigArgumentsError(String),
    #[error("Creation of directories to {} unsuccessful", .0.display())]
    ConfigCreateDirectoryError(PathBuf),
    #[error("Unexpected error: home directory not found. Consider specifying a configuration directory by setting \"RUST_STOCK_TRACKER_CONFIGURATION_DIRECTORY\"")]
    ConfigHomeDirectoryNotFoundError,
    #[error("Command string not recognized.")]
    CommandInvalidError,
    #[error("Unexpected error: attempted to login as user {0}, but user {0} was not found.")]
    StateInvalidUserError(String),
    #[error("Command attempted without logging in.")]
    StateNoUserError,
    #[error("Input not recognized.")]
    InvalidInputError,
}