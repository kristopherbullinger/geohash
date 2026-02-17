use crate::neighbors::Direction;
use crate::{Coord, GeohashError, Neighbors, Rect};
use libm::ldexp;

/// A Geohash stores the string representation of the geohash code in an array.
#[derive(Debug, Copy, Clone, Eq)]
pub struct Geohash {
    // stores the ascii characters of the geohash
    digits: [u8; 12],
    len: u8,
}

impl std::hash::Hash for Geohash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.digits[0..self.len as usize].hash(state);
    }
}

impl std::cmp::PartialEq for Geohash {
    fn eq(&self, other: &Self) -> bool {
        self.digits[0..self.len as usize] == other.digits[0..other.len as usize]
    }
}

impl std::cmp::PartialOrd for Geohash {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.digits[0..self.len as usize].partial_cmp(&other.digits[0..other.len as usize])
    }
}

impl std::cmp::Ord for Geohash {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.digits[0..self.len as usize].cmp(&other.digits[0..other.len as usize])
    }
}
impl std::fmt::Display for Geohash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = self.as_str();
        write!(f, "{}", s)
    }
}

impl Geohash {
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns the string representation of this geohash.
    /// ```rust
    /// let gh = geohash::Geohash::from_str("2hb").unwrap();
    /// assert_eq!(gh.as_str(), "2hb");
    /// ```
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.digits[0..self.len as usize]).unwrap()
    }

    /// Returns a geohash which can be used as an upper bound when
    /// querying against a sorted list of geocodes for all descendents
    /// of this geocode.
    /// ```rust
    /// let gh = geohash::Geohash::from_str("2hb").unwrap();
    /// let next = gh.lexicographic_descendants_upper_bound().unwrap();
    /// assert_eq!(next.as_str(), "2hc");
    ///
    ///
    /// let gh = geohash::Geohash::from_str("2hz").unwrap();
    /// let next = gh.lexicographic_descendants_upper_bound().unwrap();
    /// assert_eq!(next.as_str(), "2j");
    ///
    /// let gh = geohash::Geohash::from_str("zzzzzz").unwrap();
    /// let next = gh.lexicographic_descendants_upper_bound();
    /// assert_eq!(next, None);
    /// ```
    pub fn lexicographic_descendants_upper_bound(mut self) -> Option<Geohash> {
        if self.len() == 0 {
            return None;
        }

        for i in (0..self.len()).rev() {
            let current = self.digits[i];
            if current == b'z' {
                continue;
            }
            let next_digit = DECODER[current as usize] + 1;
            self.digits[i] = BASE32_CODES[next_digit as usize] as u8;
            self.len = (i + 1) as u8;
            return Some(self);
        }

        // All characters were max (e.g. "zzzz")
        None
    }

    /// Decode a geohash string representation into a Geohash.
    pub fn from_str(hash_str: &str) -> Result<Geohash, GeohashError> {
        let bytes = hash_str.as_bytes();
        if !(1..=12).contains(&bytes.len()) {
            return Err(GeohashError::InvalidLength(bytes.len()));
        }
        let mut digits = [0u8; 12];
        for (i, b) in bytes.iter().copied().enumerate() {
            if DECODER[b as usize] == 0xff {
                return Err(GeohashError::InvalidHashCharacter(b as char));
            }
            digits[i] = b;
        }
        Ok(Geohash {
            digits,
            len: bytes.len() as u8,
        })
    }

    /// Returns the geohash that covers this point at the specified
    /// zoom level. Returns error if the point or len is out of range.
    pub fn from_point_and_len(coord: Coord<f64>, len: usize) -> Result<Geohash, GeohashError> {
        encode_coord(coord, len)
    }

    /// Removes and returns the last byte.
    pub fn pop(&mut self) -> Option<u8> {
        if self.len() > 0 {
            let last = self.len() - 1;
            let out = Some(self.digits[last as usize]);
            self.len -= 1;
            out
        } else {
            None
        }
    }

    /// Add one digit to this geohash, represented as a byte. The byte
    /// value must be a valid geohash base32 byte. Returns error if the
    /// maximum length would be exceeded.
    /// ```rust
    /// let mut gh = geohash::Geohash::from_str("9q60y").unwrap();
    /// gh.push(b'4');
    /// assert_eq!(gh.as_str(), "9q60y4");
    /// ```
    pub fn push(&mut self, digit: u8) -> Result<(), GeohashError> {
        if DECODER[digit as usize] == 0xFF {
            return Err(GeohashError::InvalidHashCharacter(digit as char));
        } else if self.len() == 12 {
            return Err(GeohashError::InvalidLength(13));
        } else {
            self.digits[self.len as usize] = digit;
            self.len += 1;
            Ok(())
        }
    }

    /// Returns the bounding box that this geohash represents.
    pub fn bbox(self) -> Rect<f64> {
        decode_bbox(self)
    }

    /// Returns the neighbor of this geohash in the direction specified,
    /// if there is one.
    pub fn neighbor(self, direction: Direction) -> Option<Geohash> {
        neighbor(self, direction)
    }

    /// Returns all neighbors of this geohash at the same zoom level.
    pub fn neighbors(self) -> impl IntoIterator<Item = Geohash> {
        let n = neighbors(self);
        let Neighbors {
            sw,
            s,
            se,
            w,
            e,
            nw,
            n,
            ne,
        } = n;
        [sw, s, se, w, e, nw, n, ne].into_iter().flatten()
    }

    /// Returns the centerpoint of the geohash area.
    pub fn center(self) -> Coord<f64> {
        decode_geohash(self).0
    }

    /// Returns the center point of the geohash, along with the distances
    /// between the center and edges.
    pub fn center_and_range(self) -> (Coord<f64>, f64, f64) {
        decode_geohash(self)
    }

    /// Returns an iterator over all direct child geohashes of this one.
    /// Returns None if maximum depth would be exceeded.
    pub fn children(self) -> Option<impl IntoIterator<Item = Geohash>> {
        if self.len() < 12 {
            Some(BASE32_CODES.iter().copied().map(move |c| {
                let mut out = self;
                out.digits[out.len()] = c as u8;
                out.len += 1;
                out
            }))
        } else {
            None
        }
    }
}

// the alphabet for the base32 encoding used in geohashing
#[rustfmt::skip]
const BASE32_CODES: [char; 32] = [
    '0', '1', '2', '3', '4', '5', '6', '7',
    '8', '9', 'b', 'c', 'd', 'e', 'f', 'g',
    'h', 'j', 'k', 'm', 'n', 'p', 'q', 'r',
    's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
];

// array that is indexed into to get the value of a character in our base32 alphabet
#[rustfmt::skip]
const DECODER: [u8; 256] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,

    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
    0x08, 0x09, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,

    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,

    0xff, 0xff, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0xff, 0x11, 0x12, 0xff, 0x13, 0x14, 0xff,
    0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c,
    0x1d, 0x1e, 0x1f, 0xff, 0xff, 0xff, 0xff, 0xff,

    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,

    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,

    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,

    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];

// bit shifting functions used in encoding and decoding

// spread takes a u32 and deposits its bits into the evenbit positions of a u64
#[inline]
fn spread(x: u32) -> u64 {
    let mut new_x = x as u64;
    new_x = (new_x | (new_x << 16)) & 0x0000ffff0000ffff;
    new_x = (new_x | (new_x << 8)) & 0x00ff00ff00ff00ff;
    new_x = (new_x | (new_x << 4)) & 0x0f0f0f0f0f0f0f0f;
    new_x = (new_x | (new_x << 2)) & 0x3333333333333333;
    new_x = (new_x | (new_x << 1)) & 0x5555555555555555;

    new_x
}

// spreads the inputs, then shifts the y input and does a bitwise or to fill the remaining bits in x
#[inline]
fn interleave(x: u32, y: u32) -> u64 {
    spread(x) | (spread(y) << 1)
}

// squashes the even bit positions of a u64 into a u32
#[inline]
fn squash(x: u64) -> u32 {
    let mut new_x = x & 0x5555555555555555;
    new_x = (new_x | (new_x >> 1)) & 0x3333333333333333;
    new_x = (new_x | (new_x >> 2)) & 0x0f0f0f0f0f0f0f0f;
    new_x = (new_x | (new_x >> 4)) & 0x00ff00ff00ff00ff;
    new_x = (new_x | (new_x >> 8)) & 0x0000ffff0000ffff;
    new_x = (new_x | (new_x >> 16)) & 0x00000000ffffffff;
    new_x as u32
}

// uses the squash function to create a 32 from the even bits
// then shifts the input right and squashes to create a u32 from the odd bits
#[inline]
fn deinterleave(x: u64) -> (u32, u32) {
    (squash(x), squash(x >> 1))
}

/// Encode a coordinate to a geohash with length `len`.
///
/// ### Examples
///
/// Encoding a coordinate to a length five geohash:
///
/// ```rust
/// let coord = geohash::Coord { x: -120.6623, y: 35.3003 };
///
/// let gh = geohash::encode_coord(coord, 5).expect("Invalid coordinate");
///
/// assert_eq!(gh.as_str(), "9q60y");
/// ```
///
/// Encoding a coordinate to a length ten geohash:
///
/// ```rust
/// let coord = geohash::Coord { x: -120.6623, y: 35.3003 };
///
/// let gh = geohash::encode_coord(coord, 10).expect("Invalid coordinate");
///
/// assert_eq!(gh.as_str(), "9q60y60rhs");
/// ```
pub fn encode_coord(c: Coord<f64>, len: usize) -> Result<Geohash, GeohashError> {
    let max_lat = 90f64;
    let min_lat = -90f64;
    let max_lon = 180f64;
    let min_lon = -180f64;

    if !(min_lon..=max_lon).contains(&c.x) || !(min_lat..=max_lat).contains(&c.y) {
        return Err(GeohashError::InvalidCoordinateRange(c));
    }

    if !(1..=12).contains(&len) {
        return Err(GeohashError::InvalidLength(len));
    }

    // divides the latitude by 180, then adds 1.5 to give a value between 1 and 2
    // then we take the first 32 bits of the significand as a u32
    let lat32 = ((c.y * 0.005555555555555556 + 1.5).to_bits() >> 20) as u32;
    // same as latitude, but a division by 360 instead of 180
    let lon32 = ((c.x * 0.002777777777777778 + 1.5).to_bits() >> 20) as u32;

    let mut interleaved_int = interleave(lat32, lon32);

    let mut digits = [0u8; 12];
    // loop through and take the first 5 bits of the interleaved value ech iteration
    for i in 0..len {
        // shifts so that the high 5 bits are now the low five bits, then masks to get their value
        let code = (interleaved_int >> 59) as usize & (0x1f);
        // uses that value to index into the array of base32 codes
        digits[i] = BASE32_CODES[code] as u8;
        // shifts the interleaved bits left by 5, so we get the next 5 bits on the next iteration
        interleaved_int <<= 5;
    }
    Ok(Geohash {
        digits,
        len: len as u8,
    })
}

/// Decode geohash into latitude, longitude
///
/// Returns:
/// A four-element tuple describes a bound box:
/// * min_lat
/// * max_lat
/// * min_lon
/// * max_lon
pub fn decode_bbox(geohash: Geohash) -> Rect<f64> {
    let bits = geohash.len() * 5;

    let mut int_hash: u64 = 0;
    for c in geohash.digits[0..geohash.len()].iter().copied() {
        // getting the value from the array converts from the base32 alphabet to an integer value
        let hash_value = DECODER[c as usize];
        // shift int_hash and deposit the newly decoded bits into its lowest bits
        int_hash <<= 5;
        int_hash |= hash_value as u64;
    }

    bbox_int_with_precision(int_hash, bits as u32)
}

fn decode_range(x: u32, r: f64) -> f64 {
    // f64 in the range 1 to 2 where 1 would represent -r and 2 would represent r
    let p = f64::from_bits(((x as u64) << 20) | (1023 << 52));
    // converts the value between 1 and 2 to a value between -r and r
    2.0 * r * (p - 1.0) - r
}

fn error_with_precision(bits: u32) -> (f64, f64) {
    let lat_bits = bits / 2;
    let long_bits = bits - lat_bits;

    // the ldexp(x, n) function is equivalent to x * 2^n but with better performance
    let lat_err = ldexp(180.0, -(lat_bits as i32));
    let long_err = ldexp(360.0, -(long_bits as i32));
    (lat_err, long_err)
}

fn bbox_int_with_precision(hash: u64, bits: u32) -> Rect<f64> {
    let full_hash = hash << (64 - bits);
    let (lat_int, long_int) = deinterleave(full_hash);
    let lat = decode_range(lat_int, 90.0);
    let long = decode_range(long_int, 180.0);
    let (lat_err, long_err) = error_with_precision(bits);

    Rect::new(
        Coord { x: long, y: lat },
        Coord {
            x: long + long_err,
            y: lat + lat_err,
        },
    )
}

/// Decode a geohash into a coordinate with some longitude/latitude error. The
/// return value is `(<coordinate>, <longitude error>, <latitude error>)`.
///
/// ### Examples
///
/// Decoding a length five geohash:
///
/// ```rust
/// let geohash_str = "9q60y";
///
/// let gh = geohash::Geohash::from_str(geohash_str).expect("Invalid hash string");
/// let decoded = geohash::decode_geohash(gh);
///
/// assert_eq!(
///     decoded,
///     (
///         geohash::Coord {
///             x: -120.65185546875,
///             y: 35.31005859375,
///         },
///         0.02197265625,
///         0.02197265625,
///     ),
/// );
/// ```
///
/// Decoding a length ten geohash:
///
/// ```rust
/// let geohash_str = "9q60y60rhs";
///
/// let gh = geohash::Geohash::from_str(geohash_str).expect("Invalid hash string");
/// let decoded = geohash::decode_geohash(gh);
///
/// assert_eq!(
///     decoded,
///     (
///         geohash::Coord {
///             x: -120.66229999065399,
///             y: 35.300298035144806,
///         },
///         0.000005364418029785156,
///         0.000002682209014892578,
///     ),
/// );
/// ```
pub fn decode_geohash(geohash: Geohash) -> (Coord<f64>, f64, f64) {
    let rect = decode_bbox(geohash);
    let c0 = rect.min();
    let c1 = rect.max();
    (
        Coord {
            x: (c0.x + c1.x) / 2f64,
            y: (c0.y + c1.y) / 2f64,
        },
        (c1.x - c0.x) / 2f64,
        (c1.y - c0.y) / 2f64,
    )
}

/// Find neighboring geohashes for the given geohash and direction.
///
/// ### Examples
///
/// ```
/// # use geohash::Direction;
/// # fn main() {
/// let geohash_str = "9q60y60rhs";
/// let gh = geohash::Geohash::from_str(geohash_str).expect("Invalid hash string");
///
/// let neighbor = geohash::neighbor(gh, Direction::N).unwrap();
///
/// assert_eq!(neighbor.as_str(), "9q60y60rht");
/// # }
/// ```
pub fn neighbor(geohash: Geohash, direction: Direction) -> Option<Geohash> {
    let (coord, lon_err, lat_err) = decode_geohash(geohash);
    let (dlat, dlng) = direction.to_tuple();
    let neighbor_coord = Coord {
        x: ((coord.x + 2f64 * lon_err.abs() * dlng) + 180.0).rem_euclid(360.0) - 180.0,
        y: ((coord.y + 2f64 * lat_err.abs() * dlat) + 90.0).rem_euclid(180.0) - 90.0,
    };
    encode_coord(neighbor_coord, geohash.len()).ok()
}

/// Find all neighboring geohashes for the given geohash.
///
/// ### Examples
///
/// ```
/// let geohash_str = "9q60y60rhs";
/// let gh = geohash::Geohash::from_str(geohash_str).expect("Invalid hash string");
///
/// let neighbors = geohash::neighbors(gh);
///
/// assert_eq!(
///     neighbors,
///     geohash::Neighbors {
///         n: Some(geohash::Geohash::from_str("9q60y60rht").unwrap()),
///         ne: Some(geohash::Geohash::from_str("9q60y60rhv").unwrap()),
///         e: Some(geohash::Geohash::from_str("9q60y60rhu").unwrap()),
///         se: Some(geohash::Geohash::from_str("9q60y60rhg").unwrap()),
///         s: Some(geohash::Geohash::from_str("9q60y60rhe").unwrap()),
///         sw: Some(geohash::Geohash::from_str("9q60y60rh7").unwrap()),
///         w: Some(geohash::Geohash::from_str("9q60y60rhk").unwrap()),
///         nw: Some(geohash::Geohash::from_str("9q60y60rhm").unwrap()),
///     }
/// );
/// ```
pub fn neighbors(geohash: Geohash) -> Neighbors {
    Neighbors {
        sw: neighbor(geohash, Direction::SW),
        s: neighbor(geohash, Direction::S),
        se: neighbor(geohash, Direction::SE),
        w: neighbor(geohash, Direction::W),
        e: neighbor(geohash, Direction::E),
        nw: neighbor(geohash, Direction::NW),
        n: neighbor(geohash, Direction::N),
        ne: neighbor(geohash, Direction::NE),
    }
}

impl std::str::FromStr for Geohash {
    type Err = GeohashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Geohash::from_str(s)
    }
}
