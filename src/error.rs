use std::fmt;
use std::io;

use prelude::*;


#[derive(Debug)]
pub enum StateError {
    InconsistentState,
    DataDoesNotMatchTheChecksum,
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "::mt::StateError::{}", match *self {
            StateError::InconsistentState => "InconsistentState",
            StateError::DataDoesNotMatchTheChecksum => "DataDoesNotMatchTheChecksum",
        })
    }
}

impl ::std::error::Error for StateError {
    fn description(&self) -> &str {
        self.as_str()
    }
}

impl StateError {
    pub fn as_str(&self) -> &'static str {
        match *self {
            StateError::InconsistentState => "inconsistent state",
            StateError::DataDoesNotMatchTheChecksum => "data does not match the checksum",
        }
    }
}


// -------------------------------------------------------------------------------------------------


#[derive(Debug)]
pub enum AccessError {
    IndexIsOutOfBounds,
}

impl fmt::Display for AccessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "::mt::StateError::{}", match *self {
            AccessError::IndexIsOutOfBounds => "IndexIsOutOfBounds",
        })
    }
}

impl ::std::error::Error for AccessError {
    fn description(&self) -> &str {
        self.as_str()
    }
}

impl AccessError {
    pub fn as_str(&self) -> &'static str {
        match *self {
            AccessError::IndexIsOutOfBounds => "index is out of bounds",
        }
    }
}


// -------------------------------------------------------------------------------------------------


pub type Result<T> = ::std::result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    State(StateError),
    Access(AccessError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "::mt::Error(")?;
        match *self {
            Error::Io(ref e) => write!(f, "{}", e)?,
            Error::State(ref e) => write!(f, "{}", e)?,
            Error::Access(ref e) => write!(f, "{}", e)?,
        }
        write!(f, ")")
    }
}

impl Error {
    pub fn new_ro<M: Into<String>>(msg: M) -> Self {
        Error::Io(io::Error::new(io::ErrorKind::PermissionDenied, msg.into()))
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<StateError> for Error {
    fn from(err: StateError) -> Self {
        Error::State(err)
    }
}

impl From<AccessError> for Error {
    fn from(err: AccessError) -> Self {
        Error::Access(err)
    }
}

impl ::std::error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref e) => e.description(),
            Error::State(ref e) => e.description(),
            Error::Access(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        match *self {
            Error::Io(ref e) => e.cause(),
            Error::State(ref e) => e.cause(),
            Error::Access(ref e) => e.cause(),
        }
    }
}

impl Error {
    pub fn is_io_error(&self) -> bool {
        match *self {
            Error::Io(_) => true,
             _ => false,
        }
    }

    pub fn into_io_error(self) -> Option<io::Error> {
        match self {
            Error::Io(err) => Some(err),
            _ => None,
        }
    }

    pub fn is_state_error(&self) -> bool {
        match *self {
            Error::State(_) => true,
             _ => false,
        }
    }

    pub fn into_state_error(self) -> Option<StateError> {
        match self {
            Error::State(err) => Some(err),
            _ => None,
        }
    }

    pub fn is_access_error(&self) -> bool {
        match *self {
            Error::Access(_) => true,
             _ => false,
        }
    }

    pub fn into_access_error(self) -> Option<AccessError> {
        match self {
            Error::Access(err) => Some(err),
            _ => None,
        }
    }
}
