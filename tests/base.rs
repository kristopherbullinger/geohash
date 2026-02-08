use geohash::{Coord, Geohash, decode_geohash, encode_coord, neighbors};
use serde::Deserialize;

// struct to allow for deserialization
#[derive(Debug, Deserialize)]
struct TestCase {
    string_hash: String,
    lat: f64,
    long: f64,
}

#[test]
fn test_encode() {
    // use the testcases file to check encoding correctness
    let mut rdr =
        csv::Reader::from_path("tests/testcases.csv").expect("Failed to open file of test cases");
    let iter = rdr.deserialize();
    for result in iter {
        let record: TestCase = result.expect("Unable to deserialize record");
        let c = Coord {
            x: record.long,
            y: record.lat,
        };
        assert_eq!(encode_coord(c, 12).unwrap().as_str(), record.string_hash);
    }
    // check that errors are thrown appropriately

    // should throw an error because the length is greater than 12
    let c1 = Coord {
        x: 117f64,
        y: 32f64,
    };
    assert!(encode_coord(c1, 13).is_err());

    // should throw an error because the longitude is out of range
    let c2 = Coord {
        x: 190f64,
        y: -80f64,
    };
    assert!(encode_coord(c2, 3usize).is_err());

    // should throw an error because the latitude is out of range
    let c3 = Coord {
        x: 100f64,
        y: -100f64,
    };
    assert!(encode_coord(c3, 3usize).is_err());

    // should throw an error because the longitude is NAN
    let c4 = Coord {
        x: f64::NAN,
        y: 50f64,
    };
    assert!(encode_coord(c4, 4usize).is_err());

    // should throw an error because the latitude is NAN
    let c5 = Coord {
        x: 100f64,
        y: f64::NAN,
    };
    assert!(encode_coord(c5, 4usize).is_err());
}

fn compare_within(a: f64, b: f64, diff: f64) {
    assert!(
        (a - b).abs() < diff,
        "{:?} and {:?} should be within {:?}",
        a,
        b,
        diff
    );
}

fn compare_decode(gh: &str, exp_lon: f64, exp_lat: f64, exp_lon_err: f64, exp_lat_err: f64) {
    let (coord, lon_err, lat_err) = decode_geohash(Geohash::from_str(gh).unwrap());
    let diff = 1e-5f64;
    compare_within(lon_err, exp_lon_err, diff);
    compare_within(lat_err, exp_lat_err, diff);
    compare_within(coord.x, exp_lon, diff);
    compare_within(coord.y, exp_lat, diff);
}

#[test]
fn test_decode() {
    // test decodes against the test cases file
    let diff = 1e-5f64;
    let mut rdr =
        csv::Reader::from_path("tests/testcases.csv").expect("Failed to open file of test cases");
    let iter = rdr.deserialize();
    for result in iter {
        let record: TestCase = result.expect("Unable to deserialize record");
        let gh = Geohash::from_str(&record.string_hash).unwrap();
        let c = decode_geohash(gh);
        compare_within(c.0.x, record.long, diff);
        compare_within(c.0.y, record.lat, diff);
    }

    // run these as well, since the test against the csv doesn't testt he error ranges
    compare_decode("ww8p1r4t8", 112.558386, 37.832386, 0.000021457, 0.000021457);
    compare_decode("9g3q", -99.31640625, 19.423828125, 0.17578125, 0.087890625);

    // check for errors being thrown appropriately

    // should throw an error since a is not a valid character
    assert!(Geohash::from_str("abcd").is_err());

    // should throw an error since the input is too long
    assert!(Geohash::from_str("ww8p1r4t8ww8p1r4t8").is_err());
}

#[test]
fn test_neighbor() {
    let gh = Geohash::from_str("ww8p1r4t8").unwrap();
    let ns = neighbors(gh);
    assert_eq!(ns.sw.as_ref().map(Geohash::as_str), Some("ww8p1r4mr"));
    assert_eq!(ns.s.as_ref().map(Geohash::as_str), Some("ww8p1r4t2"));
    assert_eq!(ns.se.as_ref().map(Geohash::as_str), Some("ww8p1r4t3"));
    assert_eq!(ns.w.as_ref().map(Geohash::as_str), Some("ww8p1r4mx"));
    assert_eq!(ns.e.as_ref().map(Geohash::as_str), Some("ww8p1r4t9"));
    assert_eq!(ns.nw.as_ref().map(Geohash::as_str), Some("ww8p1r4mz"));
    assert_eq!(ns.n.as_ref().map(Geohash::as_str), Some("ww8p1r4tb"));
    assert_eq!(ns.ne.as_ref().map(Geohash::as_str), Some("ww8p1r4tc"));

    let gh = Geohash::from_str("2hb").unwrap();
    let ns = neighbors(gh);
    assert_eq!(ns.sw.as_ref().map(Geohash::as_str), Some("rux"));
    assert_eq!(ns.s.as_ref().map(Geohash::as_str), Some("2h8"));
    assert_eq!(ns.se.as_ref().map(Geohash::as_str), Some("2h9"));
    assert_eq!(ns.w.as_ref().map(Geohash::as_str), Some("ruz"));
    assert_eq!(ns.e.as_ref().map(Geohash::as_str), Some("2hc"));
    assert_eq!(ns.nw.as_ref().map(Geohash::as_str), Some("rvp"));
    assert_eq!(ns.n.as_ref().map(Geohash::as_str), Some("2j0"));
    assert_eq!(ns.ne.as_ref().map(Geohash::as_str), Some("2j1"));
}

#[test]
fn test_neighbor_wide() {
    let gh = Geohash::from_str("9g3m").unwrap();
    let ns = neighbors(gh);
    assert_eq!(ns.sw.as_ref().map(|gh| gh.as_str()), Some("9g3h"));
    assert_eq!(ns.s.as_ref().map(|gh| gh.as_str()), Some("9g3k"));
    assert_eq!(ns.se.as_ref().map(|gh| gh.as_str()), Some("9g3s"));
    assert_eq!(ns.w.as_ref().map(|gh| gh.as_str()), Some("9g3j"));
    assert_eq!(ns.e.as_ref().map(|gh| gh.as_str()), Some("9g3t"));
    assert_eq!(ns.nw.as_ref().map(|gh| gh.as_str()), Some("9g3n"));
    assert_eq!(ns.n.as_ref().map(|gh| gh.as_str()), Some("9g3q"));
    assert_eq!(ns.ne.as_ref().map(|gh| gh.as_str()), Some("9g3w"));
}
