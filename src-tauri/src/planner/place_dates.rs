use std::collections::HashMap;

use chrono::NaiveDateTime;

use crate::{
    models::DayLayout,
    trips::{day_layout_for_span, CHOSEN_DATE_FORMAT},
};

/// Day folders follow the span widened by the real file dates, so every batch of
/// one suggestion gets the same layout.
pub(super) fn day_layout_for(
    file_ids: &[String],
    span: &[&String],
    dates: &HashMap<&str, NaiveDateTime>,
) -> Result<DayLayout, String> {
    let mut all_dates = span
        .iter()
        .map(|date| parse_date(date))
        .collect::<Result<Vec<_>, _>>()?;
    all_dates.extend(file_ids.iter().filter_map(|id| dates.get(id.as_str())));
    let first = all_dates
        .iter()
        .min()
        .ok_or("A place folder needs dates.")?;
    let last = all_dates
        .iter()
        .max()
        .ok_or("A place folder needs dates.")?;
    Ok(day_layout_for_span(first.date(), last.date()))
}

pub(super) fn parse_date(date: &str) -> Result<NaiveDateTime, String> {
    NaiveDateTime::parse_from_str(date, CHOSEN_DATE_FORMAT)
        .map_err(|error| format!("Invalid place suggestion date {date}: {error}"))
}
