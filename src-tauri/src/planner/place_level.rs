use std::collections::BTreeSet;

use chrono::{Datelike, NaiveDate};

/// The date folder a country or place folder sits in: the one its files span.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PlaceLevel {
    /// Files in more than one month: `<year>/<folder>/<month>/<day>`.
    Year,
    /// Files on more than one day of one month: `<year>/<month>/<folder>/<day>`.
    Month,
    /// Files on one day: `<year>/<month>/<day>/<folder>`.
    Day,
}

/// The days a folder's files were taken on, within one year.
#[derive(Debug, Default)]
pub(super) struct DaySpan(BTreeSet<(u32, u32)>);

impl DaySpan {
    pub(super) fn add(&mut self, date: NaiveDate) {
        self.0.insert((date.month(), date.day()));
    }

    pub(super) fn level(&self) -> PlaceLevel {
        let months: BTreeSet<u32> = self.0.iter().map(|(month, _)| *month).collect();
        if months.len() > 1 {
            PlaceLevel::Year
        } else if self.0.len() > 1 {
            PlaceLevel::Month
        } else {
            PlaceLevel::Day
        }
    }
}
