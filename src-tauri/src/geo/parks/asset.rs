//! Parses the gzipped park outlines that `scripts/geodata/build_parks.py` builds.
//!
//! Each line is tab separated: `kind name area_km2 lat lon rings`. Rings are
//! polyline strings separated by spaces.

use std::io::Read;

use flate2::read::GzDecoder;

use super::outline::Outline;
use super::polyline::decode;
use super::Park;
use crate::geo::distance::Coordinates;
use crate::geo::kind::PlaceKind;

pub(super) fn parse(gzipped: &[u8]) -> Vec<Park> {
    let mut text = String::new();
    GzDecoder::new(gzipped)
        .read_to_string(&mut text)
        .expect("the bundled parks asset is valid gzip");
    text.lines()
        .map(|line| parse_park(line).unwrap_or_else(|| panic!("bad parks line: {line:.80}")))
        .collect()
}

fn parse_park(line: &str) -> Option<Park> {
    let mut fields = line.split('\t');
    let kind = PlaceKind::from_tag(fields.next()?)?;
    let name = fields.next()?.to_owned();
    let area_km2 = fields.next()?.parse().ok()?;
    let latitude = fields.next()?.parse().ok()?;
    let longitude = fields.next()?.parse().ok()?;
    let rings = fields
        .next()?
        .split(' ')
        .map(decode)
        .collect::<Option<Vec<_>>>()?;
    Some(Park {
        name,
        kind,
        area_km2,
        label: Coordinates::new(latitude, longitude),
        outline: Outline::new(rings)?,
    })
}
