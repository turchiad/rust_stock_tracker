use crate::ProjectError;
use crate::ProjectError::*;
use std::fmt; // So we may define `Display` for `Command`

/// The 'UserCommand' enum represents the variety of input cases relating to users
#[derive(Debug, Clone)]
pub enum UserCommand {
    Create,
    Delete,
    Login,
    Logout,
    Showall,
}

/// The 'StockCommand' enum represents the variety of input cases relating to stocks
#[derive(Debug, Clone)]
pub enum StockCommand {
    Buy,
    Create,
    Delete,
}

/// The `Command` enum represents the variety of input cases a user could specify.
#[derive(Debug, Clone)]
pub enum Command {
    Init,
    Console,
    Exit, // Only accessible in console mode 
    UserC(UserCommand),
    StockC(StockCommand),
}


impl Command {

    /// Constructor for the `Command` enum to parse a `String` input
    pub fn new(s: &str) -> Result<Command, ProjectError> {
        Ok(match String::from(s).to_lowercase().as_str() {
            // Special Commands
            "i" | "init"            => Command::Init,
            "co" | "console"        => Command::Console,
            "q" | "quit" | "exit"   => Command::Exit,
            // Zero State Commands
            "cu" | "create-user"    => Command::UserC(UserCommand::Create),
            "du" | "delete-user"    => Command::UserC(UserCommand::Delete),
            "li" | "login"          => Command::UserC(UserCommand::Login),
            "lo" | "logout"         => Command::UserC(UserCommand::Logout),
            "sa" | "showall"        => Command::UserC(UserCommand::Showall),
            "cs" | "create-stock"   => Command::StockC(StockCommand::Create),
            "ds" | "delete-stock"   => Command::StockC(StockCommand::Delete),
            // Logged In Commands
            "bs" | "buy-stock"      => Command::StockC(StockCommand::Buy),
            _ => return Err(CommandInvalidError),
        })
    }

    /// Returns the number of arguments expected after the `Command`
    pub fn num_args(&self) -> i32 {
        match self {
            // Special Commands
            Command::Init                           => 0,
            Command::Console                        => 0,
            Command::Exit                           => 0,
            // Zero State Commands
            Command::UserC(UserCommand::Create)     => 1,
            Command::UserC(UserCommand::Delete)     => 1,
            Command::UserC(UserCommand::Login)      => 1,
            Command::UserC(UserCommand::Logout)     => 0,
            Command::UserC(UserCommand::Showall)    => 0,
            Command::StockC(StockCommand::Create)   => 1,
            Command::StockC(StockCommand::Delete)   => 1,
            // Logged In Commands
            Command::StockC(StockCommand::Buy)      => 2,
        }
    }
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self{
            // Special Commands
            Command::Init                           => "init",
            Command::Console                        => "console",
            Command::Exit                           => "exit",
            // Zero State Commands
            Command::UserC(UserCommand::Create)     => "create-user",
            Command::UserC(UserCommand::Delete)     => "delete-user",
            Command::UserC(UserCommand::Login)      => "login",
            Command::UserC(UserCommand::Logout)     => "logout",
            Command::UserC(UserCommand::Showall)    => "showall",
            Command::StockC(StockCommand::Create)   => "create-stock",
            Command::StockC(StockCommand::Delete)   => "delete-stock",
            // Logged In Commands
            Command::StockC(StockCommand::Buy)      => "buy-stock"
        })
    }
}