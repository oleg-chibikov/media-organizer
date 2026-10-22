use chrono::NaiveDate;

use crate::{models::DayLayout, trips::day_layout::day_layout_for_span};

fn date(text: &str) -> NaiveDate {
    NaiveDate::parse_from_str(text, "%Y-%m-%d").expect("a date")
}

#[test]
fn one_day_needs_no_day_folder() {
    let day = date("2025-10-28");
    assert_eq!(day_layout_for_span(day, day), DayLayout::Single);
}

#[test]
fn ten_days_across_a_month_boundary_use_month_day_folders() {
    let layout = day_layout_for_span(date("2025-10-27"), date("2025-11-06"));
    assert_eq!(layout, DayLayout::MonthDay);
}

#[test]
fn thirty_one_days_still_use_month_day_folders() {
    let layout = day_layout_for_span(date("2025-10-01"), date("2025-11-01"));
    assert_eq!(layout, DayLayout::MonthDay);
}

#[test]
fn forty_days_use_month_then_day_folders() {
    let layout = day_layout_for_span(date("2025-10-01"), date("2025-11-10"));
    assert_eq!(layout, DayLayout::MonthFolderDay);
}
