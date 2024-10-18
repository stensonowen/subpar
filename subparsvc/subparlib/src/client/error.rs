
use hyper;
use protobuf::ProtobufError;

use std::error::Error as StdError;
use std::fmt;

pub type FetchResult<T> = Result<T, FetchError>;

#[derive(Debug)]
pub enum FetchError {
    RequestError(hyper::Error),
    ParseError(ProtobufError),
}

impl StdError for FetchError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        use FetchError::*;
        match self {
            RequestError(e) => Some(e),
            ParseError(e) => Some(e),
        }
    }
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FetchError::*;
        match self {
            RequestError(e) => write!(f, "Error retrieving data: {}", e),
            ParseError(e) =>   write!(f, "Error parsing data: {}", e),
        }
    }
}

impl From<hyper::Error> for FetchError {
    fn from(err: hyper::Error) -> Self {
        FetchError::RequestError(err)
    }
}

impl From<ProtobufError> for FetchError {
    fn from(err: ProtobufError) -> Self {
        FetchError::ParseError(err)
    }
}




