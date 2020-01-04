use super::error::Error;
//------------------------------------------------------------------------------
use std::io::Write;

//------------------------------------------------------------------------------
pub type Result<R> = std::result::Result<R, Error>;

//------------------------------------------------------------------------------
pub fn get<S>(option: Option<S>) -> Result<S> {
    match option {
        Some(value) => Ok(value),
        None => Err(Error::None()),
    }
}

//------------------------------------------------------------------------------
pub fn handle_errors<R>(result: Result<R>) {
    if let Err(error) = result {
        match writeln!(std::io::stderr(), "{0}", error) {
            _ => (), // Do nothing
        }
    }
}
