use super::path;
use super::status;
//------------------------------------------------------------------------------
use std::io;
use std::fmt;

//------------------------------------------------------------------------------
pub type PathSendError =
    std::sync::mpsc::SendError<std::option::Option<path::PathBuf>>;
pub type StatusSendError = std::sync::mpsc::SendError<(
    std::string::String,
    status::Status,
    std::string::String,
)>;
pub type RecvError = std::sync::mpsc::RecvError;
pub type ThreadError = std::boxed::Box<dyn std::any::Any + std::marker::Send>;

//------------------------------------------------------------------------------
// Error
//------------------------------------------------------------------------------
#[derive(Debug)]
pub enum Error {
    None(),
    IO(io::Error),
    PathSend(PathSendError),
    StatusSend(StatusSendError),
    Recv(RecvError),
    Regex(regex::Error),
    Thread(ThreadError),
    StripPrefix(path::StripPrefixError),
    RelativeToRepo(),
    Infallible(std::convert::Infallible),
    UnableToParseStatus,
}

//------------------------------------------------------------------------------
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error")
    }
}

//------------------------------------------------------------------------------
impl From<PathSendError> for Error {
    fn from(error: PathSendError) -> Self {
        Error::PathSend(error)
    }
}

//------------------------------------------------------------------------------
impl From<StatusSendError> for Error {
    fn from(error: StatusSendError) -> Self {
        Error::StatusSend(error)
    }
}

//------------------------------------------------------------------------------
impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::IO(error)
    }
}

//------------------------------------------------------------------------------
impl From<RecvError> for Error {
    fn from(error: RecvError) -> Self {
        Error::Recv(error)
    }
}

//------------------------------------------------------------------------------
impl From<regex::Error> for Error {
    fn from(error: regex::Error) -> Self {
        Error::Regex(error)
    }
}

//------------------------------------------------------------------------------
impl From<ThreadError> for Error {
    fn from(error: ThreadError) -> Self {
        Error::Thread(error)
    }
}

//------------------------------------------------------------------------------
impl From<path::StripPrefixError> for Error {
    fn from(error: path::StripPrefixError) -> Self {
        Error::StripPrefix(error)
    }
}

//------------------------------------------------------------------------------
impl From<std::convert::Infallible> for Error {
    fn from(error: std::convert::Infallible) -> Self {
        Error::Infallible(error)
    }
}

