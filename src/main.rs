//! #stock_tracker
//!
//! This is the runtime for the rust_stock_tracker project

use std::env; // To allow access of CLI arguments
use std::process; // So the program may be terminated early
use stock_tracker::Config; // To allow use of the `Config` type

fn main() {
    // Process arguments
    let config = match Config::new(env::args()) {
        Ok(x) => x,
            Err(x) => {
            eprintln!("Problem parsing arguments: {}", x);
            process::exit(1);
        }
    };

    // Program Logic
    if let Err(e) = stock_tracker::run(config) {
        eprintln!("Application error: {}", e);
        process::exit(1);
    }
}
