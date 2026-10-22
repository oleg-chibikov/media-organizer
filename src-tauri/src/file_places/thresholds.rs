//! Every number place detection decides by.

use chrono::TimeDelta;

/// Home needs this many distinct days in one city.
pub(super) const HOME_MIN_DAYS: usize = 14;
/// A home learned from a scan and saved for later scans needs this many days,
/// plus the same share as above.
pub(super) const LEARNED_HOME_MIN_DAYS: usize = 60;
/// Home also needs this share, in percent, of all geotagged days.
pub(super) const HOME_MIN_DAY_SHARE_PERCENT: usize = 30;
/// A file with no GPS joins the place of a geotagged file from the same device this close
/// in time.
pub(super) const UNTAGGED_MAX_GAP: TimeDelta = TimeDelta::hours(3);
/// Points closer than this, in degrees, share one geocoder lookup.
pub(super) const GEOCODE_CACHE_STEP_DEGREES: f64 = 0.001;
