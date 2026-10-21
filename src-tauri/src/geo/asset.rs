//! Parses the gzipped GeoNames extract that `scripts/geodata` builds.
//!
//! Lines are tab separated. `C code name` adds a country. `A name` adds an
//! admin1 region. Any other line is a place:
//! `kind lat lon country_code admin1_index name population`.

use std::collections::HashMap;
use std::io::Read;

use flate2::read::GzDecoder;

use super::index::{Entry, GeoIndex, Tables, NO_ADMIN1};
use super::kind::PlaceKind;

pub(super) fn parse(gzipped: &[u8]) -> GeoIndex {
    let mut text = String::new();
    GzDecoder::new(gzipped)
        .read_to_string(&mut text)
        .expect("the bundled geodata asset is valid gzip");
    let mut tables = Tables::default();
    let mut country_ids: HashMap<&str, u16> = HashMap::new();
    for line in text.lines() {
        let mut fields = line.split('\t');
        match fields.next() {
            Some("C") => {
                let code = fields.next().unwrap_or_default();
                let name = fields.next().unwrap_or_default();
                country_ids.insert(code, tables.countries.len() as u16);
                tables.countries.push((code.into(), name.into()));
            }
            Some("A") => tables.admin1.push(fields.next().unwrap_or_default().into()),
            Some(tag) => {
                let entry = parse_place(tag, fields, &country_ids, &mut tables.names);
                tables
                    .entries
                    .push(entry.unwrap_or_else(|| panic!("bad geodata line: {line}")));
            }
            None => {}
        }
    }
    GeoIndex::new(tables)
}

fn parse_place<'a>(
    tag: &str,
    mut fields: impl Iterator<Item = &'a str>,
    country_ids: &HashMap<&str, u16>,
    names: &mut String,
) -> Option<Entry> {
    let kind = PlaceKind::from_tag(tag)?;
    let latitude = fields.next()?.parse().ok()?;
    let longitude = fields.next()?.parse().ok()?;
    let country = *country_ids.get(fields.next()?)?;
    let admin1 = fields.next()?.parse().unwrap_or(NO_ADMIN1);
    let name = fields.next()?;
    let population = fields.next()?.parse().ok()?;
    let name_start = names.len() as u32;
    names.push_str(name);
    Some(Entry {
        latitude,
        longitude,
        kind,
        country,
        admin1,
        population,
        name_start,
        name_len: u16::try_from(name.len()).ok()?,
    })
}
