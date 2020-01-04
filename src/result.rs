use super::error::Error;

//------------------------------------------------------------------------------
pub type Result<R> = std::result::Result<R, Error>;

//------------------------------------------------------------------------------
pub fn get<S>(option: Option<S>) -> Result<S> {
    match option {
        Some(value) => Ok(value),
        None => Err(Error::None()),
    }
}
