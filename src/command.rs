//! #command
//!
//! This holds the `Command` type and related methods

use crate::ProjectError;
use crate::ProjectError::*;
use std::fmt; // So we may define `Display` for `Command`

/// `StateCommand` represents commands that relate to the state, such as logging in or out.
#[derive(Debug, Clone)]
pub enum StateCommand {
    Login,
    Logout,
}

/// `UserCommand` represents commands that relate to `User` management, such as creating or deleting `User`s.
#[derive(Debug, Clone)]
pub enum UserCommand {
    Create,
    Delete,
    Edit,
    List,
}

/// `StockCommand` represents commands that relate to `Stock` management, such as creating or deleting `Stock`s
#[derive(Debug, Clone)]
pub enum StockCommand {
    Create,
    Delete,
    Edit,
    List,
}

/// `PortfolioCommand` represents commands that relate to management of the logged in user's `portfolio` of `StockUnit`s
#[derive(Debug, Clone)]
pub enum PortfolioCommand {
    Buy,
    List,
}

/// The `Command` enum represents the variety of input cases a user could specify.
#[derive(Debug, Clone)]
pub enum Command {
    Init,
    Console,
    Exit, // Only accessible in console mode 
    StateC(StateCommand),
    UserC(UserCommand),
    StockC(StockCommand),
    PortfolioC(PortfolioCommand),
}


impl Command {

    /// Constructor for the `Command` enum to parse a `String` input
    pub fn new(s: &str) -> Result<Command, ProjectError> {
        Ok(match String::from(s).to_lowercase().as_str() {
            // Special Commands
            "i" | "init"                => Command::Init,
            "co" | "console"            => Command::Console,
            "q" | "quit" | "exit"       => Command::Exit,
            // State Management Commands
            "li" | "login"              => Command::StateC(StateCommand::Login),
            "lo" | "logout"             => Command::StateC(StateCommand::Logout),
            // User Management Commands
            "cu" | "create-user"        => Command::UserC(UserCommand::Create),
            "du" | "delete-user"        => Command::UserC(UserCommand::Delete),
            "eu" | "edit-user"          => Command::UserC(UserCommand::Edit),
            "lu" | "list-users"         => Command::UserC(UserCommand::List),
            // Stock Management Commands
            "cs" | "create-stock"       => Command::StockC(StockCommand::Create),
            "ds" | "delete-stock"       => Command::StockC(StockCommand::Delete),
            "es" | "edit-stock"         => Command::StockC(StockCommand::Edit),
            "ls" | "list-stocks"        => Command::StockC(StockCommand::List), 
            // Portfolio Management Commands
            "bs" | "buy-stock"          => Command::PortfolioC(PortfolioCommand::Buy),
            "lp" | "list-portfolio"     => Command::PortfolioC(PortfolioCommand::List),
            _ => return Err(CommandInvalidError),
        })
    }

    /// Returns the number of arguments expected after the `Command`
    pub fn num_args(&self) -> i32 {
        match self {
            // Special Commands
            Command::Init                                   => 0,
            Command::Console                                => 0,
            Command::Exit                                   => 0,
            // State Management Commands
            Command::StateC(StateCommand::Login)            => 1,
            Command::StateC(StateCommand::Logout)           => 0,
            // User Management Commands
            Command::UserC(UserCommand::Create)             => 1,
            Command::UserC(UserCommand::Delete)             => 1,
            Command::UserC(UserCommand::Edit)               => 3,
            Command::UserC(UserCommand::List)               => 0,
            // Stock Management Commands
            Command::StockC(StockCommand::Create)           => 1,
            Command::StockC(StockCommand::Delete)           => 1,
            Command::StockC(StockCommand::Edit)             => 3,
            Command::StockC(StockCommand::List)             => 0,
            // Portfolio Management Commands
            Command::PortfolioC(PortfolioCommand::Buy)      => 2,
            Command::PortfolioC(PortfolioCommand::List)     => 0,
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self{
            // Special Commands
            Command::Init                                   => "init",
            Command::Console                                => "console",
            Command::Exit                                   => "exit",
            // State Management Commands
            Command::StateC(StateCommand::Login)            => "login",
            Command::StateC(StateCommand::Logout)           => "logout",
            // User Management Commands
            Command::UserC(UserCommand::Create)             => "create-user",
            Command::UserC(UserCommand::Delete)             => "delete-user",
            Command::UserC(UserCommand::Edit)               => "edit-user",
            Command::UserC(UserCommand::List)               => "list-users",
            // Stock Management Commands
            Command::StockC(StockCommand::Create)           => "create-stock",
            Command::StockC(StockCommand::Delete)           => "delete-stock",
            Command::StockC(StockCommand::Edit)             => "edit-stock",
            Command::StockC(StockCommand::List)             => "list-stocks",
            // Portfolio Management Commands
            Command::PortfolioC(PortfolioCommand::Buy)      => "buy-stock",
            Command::PortfolioC(PortfolioCommand::List)     => "list-portfolio",
        })
    }
}