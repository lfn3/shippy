use std::borrow::Borrow;
use std::env;
use shippy::err::CliError;

extern crate shippy;

fn main() -> Result<(), CliError<'static>> {
    let cwd = env::current_dir().map_err(| e |
        CliError::Io("Could not get current_dir", e))?;
    let args : Vec<String> = env::args().collect();

    Ok(())
}