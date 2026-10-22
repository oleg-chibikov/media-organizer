//! Names a trip after the countries it covers.

use std::collections::HashMap;

const UNKNOWN_COUNTRY_TITLE: &str = "Trip";

/// The country with the most files, or the top two joined with " & ".
/// Three or more countries give the top one and " & more".
pub(super) fn trip_title<'a>(file_countries: impl Iterator<Item = &'a str>) -> String {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for country in file_countries {
        *counts.entry(country).or_insert(0) += 1;
    }
    let mut ranked: Vec<(&str, usize)> = counts.into_iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    match ranked.as_slice() {
        [] => UNKNOWN_COUNTRY_TITLE.to_owned(),
        [(only, _)] => (*only).to_owned(),
        [(first, _), (second, _)] => format!("{first} & {second}"),
        [(first, _), ..] => format!("{first} & more"),
    }
}
