use super::sanitize_for_windows_filename;

/// `<place>, <country>` for a file name, or one of them when the other is empty or the same.
pub(crate) fn place_label(place: &str, country: &str) -> Option<String> {
    let place = place.trim();
    let country = country.trim();
    let label = match (place.is_empty(), country.is_empty()) {
        (true, true) => return None,
        (false, true) => place.to_owned(),
        (true, false) => country.to_owned(),
        _ if place == country => place.to_owned(),
        _ => format!("{place}, {country}"),
    };
    Some(sanitize_for_windows_filename(&label))
}
