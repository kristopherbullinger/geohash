use crate::core::Geohash;
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Neighbors {
    pub sw: Option<Geohash>,
    pub s: Option<Geohash>,
    pub se: Option<Geohash>,
    pub w: Option<Geohash>,
    pub e: Option<Geohash>,
    pub nw: Option<Geohash>,
    pub n: Option<Geohash>,
    pub ne: Option<Geohash>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// North
    N,
    /// North-east
    NE,
    /// East
    E,
    /// South-east
    SE,
    /// South
    S,
    /// South-west
    SW,
    /// West
    W,
    /// North-west
    NW,
}

impl Direction {
    pub fn to_tuple(self) -> (f64, f64) {
        match self {
            Direction::SW => (-1f64, -1f64),
            Direction::S => (-1f64, 0f64),
            Direction::SE => (-1f64, 1f64),
            Direction::W => (0f64, -1f64),
            Direction::E => (0f64, 1f64),
            Direction::NW => (1f64, -1f64),
            Direction::N => (1f64, 0f64),
            Direction::NE => (1f64, 1f64),
        }
    }
}
