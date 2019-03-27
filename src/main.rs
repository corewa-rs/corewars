extern crate corewa_rs;

use std::error::Error;

use corewa_rs::cli;

fn main() -> Result<(), Box<dyn Error>> {
    cli::run()
}
