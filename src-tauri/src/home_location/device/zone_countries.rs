//! Reads the country of a time zone from the system's `zone.tab`.

use std::fs;

const ZONE_TAB: &str = "/usr/share/zoneinfo/zone.tab";

/// The ISO country code `zone.tab` lists for `zone`, when the file is there.
pub(super) fn country_of_zone(zone: &str) -> Option<String> {
    let table = fs::read_to_string(ZONE_TAB).ok()?;
    country_in_table(&table, zone).map(str::to_owned)
}

/// Lines are `AU<TAB>-3352+15113<TAB>Australia/Sydney<TAB>comment`.
pub(crate) fn country_in_table<'a>(table: &'a str, zone: &str) -> Option<&'a str> {
    table
        .lines()
        .filter(|line| !line.starts_with('#'))
        .find_map(|line| {
            let mut columns = line.split('\t');
            let code = columns.next()?;
            (columns.nth(1)? == zone).then_some(code)
        })
}
