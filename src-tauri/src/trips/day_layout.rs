//! How a place folder splits its files into day folders.

use chrono::NaiveDate;

use super::thresholds::MONTH_DAY_MAX_SPAN_DAYS;
use crate::models::DayLayout;

/// One day needs no day folder. Up to a month uses `MM-DD`, which sorts across
/// a month boundary. Longer spans use `<month>/<day>`.
pub(crate) fn day_layout_for_span(first: NaiveDate, last: NaiveDate) -> DayLayout {
    let span_days = (last - first).num_days().abs();
    if span_days == 0 {
        DayLayout::Single
    } else if span_days <= MONTH_DAY_MAX_SPAN_DAYS {
        DayLayout::MonthDay
    } else {
        DayLayout::MonthFolderDay
    }
}
