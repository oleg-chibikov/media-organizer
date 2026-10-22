//! Every number the place grouping decides by.

use chrono::TimeDelta;

/// Farther than this from home counts as away.
pub(super) const TRIP_MIN_DISTANCE_KM: f64 = 100.0;
/// A longer pause between away files starts a new trip.
pub(super) const TRIP_MAX_GAP: TimeDelta = TimeDelta::days(3);
/// Fewer files make no trip, place folder or outing.
pub(super) const MIN_GROUP_FILES: usize = 5;
/// Home needs this many distinct days in one city.
pub(super) const HOME_MIN_DAYS: usize = 14;
/// Home also needs this share, in percent, of all geotagged days.
pub(super) const HOME_MIN_DAY_SHARE_PERCENT: usize = 30;
/// A file with no GPS joins the trip place of a geotagged file this close in time.
pub(super) const TRIP_UNTAGGED_MAX_GAP: TimeDelta = TimeDelta::hours(3);
/// A file with no GPS joins an outing with a geotagged file this close on the same day.
pub(super) const OUTING_UNTAGGED_MAX_GAP: TimeDelta = TimeDelta::hours(1);
/// Longer spans switch day folders from `MM-DD` to `<month>/<day>`.
pub(super) const MONTH_DAY_MAX_SPAN_DAYS: i64 = 31;
/// Points closer than this, in degrees, share one geocoder lookup.
pub(super) const GEOCODE_CACHE_STEP_DEGREES: f64 = 0.001;
