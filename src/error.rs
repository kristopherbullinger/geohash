use std::error::Error;
use std::fmt;

use crate::Coord;

#[derive(Debug)]
pub enum GeohashError {
    InvalidHashCharacter(char),
    InvalidCoordinateRange(Coord<f64>),
    InvalidLength(usize),
}

impl fmt::Display for GeohashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GeohashError::InvalidHashCharacter(c) => write!(f, "invalid hash character: {}", c),
            GeohashError::InvalidCoordinateRange(c) => {
                write!(f, "invalid coordinate range: {:?}", c)
            }
            GeohashError::InvalidLength(len) => write!(
                f,
                "Invalid length specified: {}. Accepted values are between 1 and 12, inclusive",
                len
            ),
        }
    }
}

impl Error for GeohashError {}
