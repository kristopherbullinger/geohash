#![doc(html_root_url = "https://docs.rs/geohash/")]

//! # Geohash
//!
//! Geohash algorithm implementation in Rust. It encodes/decodes a
//! longitude-latitude tuple into/from a hashed string. You can find
//! more about geohash algorithm on [Wikipedia](https://en.wikipedia.org/wiki/Geohash)
//!
//! ## Usage
//! ```rust
//! extern crate geohash;
//!
//! use std::error::Error;
//!
//! use geohash::{encode_coord, decode_geohash, neighbor, Direction, Coord, Geohash};
//!
//! fn main() {
//!   // encode a coordinate
//!   let c = Coord { x: 112.5584f64, y: 37.8324f64 };
//!   println!("encoding 37.8324, 112.5584: {}", encode_coord(c, 9usize).unwrap());
//!
//!   // decode a geohash
//!   let gh = Geohash::from_str("ww8p1r4t8").unwrap();
//!   let (c, _, _) = decode_geohash(gh);
//!   println!("decoding ww8p1r4t8 to: {}, {}", c.y, c.x);
//!
//!   // find a neighboring hash
//!   let gh = Geohash::from_str("ww8p1r4t8").unwrap();
//!   let sw = neighbor(gh, Direction::SW).unwrap();
//! }
//! ```
//!

mod core;
mod error;
mod neighbors;

pub use crate::core::{Geohash, decode_bbox, decode_geohash, encode_coord, neighbor, neighbors};
pub use crate::error::GeohashError;
pub use crate::neighbors::{Direction, Neighbors};
pub use geo_types::{Coord, Rect};
