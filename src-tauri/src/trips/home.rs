//! Guesses home: the city where most days were photographed.

use std::collections::{HashMap, HashSet};

use chrono::NaiveDate;

use super::{
    place_table::{PlaceId, PlaceTable},
    shots::Shot,
    thresholds::{HOME_MIN_DAYS, HOME_MIN_DAY_SHARE_PERCENT, LEARNED_HOME_MIN_DAYS},
};
use crate::geo::{Coordinates, Place};

/// The city with the most distinct geotagged days.
#[derive(Debug, Clone)]
pub(crate) struct HomeCity {
    pub(crate) place: Place,
    pub(crate) days: usize,
}

impl HomeCity {
    pub(crate) fn coordinates(&self) -> Coordinates {
        Coordinates::new(self.place.latitude, self.place.longitude)
    }

    /// Enough days to keep as home for later scans, such as one month folder.
    pub(crate) fn worth_saving(&self) -> bool {
        self.days >= LEARNED_HOME_MIN_DAYS
    }
}

/// The city with the most distinct geotagged days, when it has enough of them.
/// `shots` must be in time order.
pub(super) fn infer_home(shots: &[Shot], places: &PlaceTable) -> Option<HomeCity> {
    let mut all_days: HashSet<NaiveDate> = HashSet::new();
    // Per city: the last day counted and the number of distinct days.
    let mut city_days: HashMap<PlaceId, (NaiveDate, usize)> = HashMap::new();
    for shot in shots {
        let Some(location) = &shot.location else {
            continue;
        };
        let day = shot.taken.date();
        all_days.insert(day);
        let Some(city) = location.resolved.city else {
            continue;
        };
        let (last_day, days) = city_days.entry(city).or_insert((day, 0));
        if *days == 0 || *last_day != day {
            *last_day = day;
            *days += 1;
        }
    }
    let (city, days) = city_days
        .into_iter()
        .map(|(city, (_, days))| (city, days))
        .max_by(|a, b| a.1.cmp(&b.1).then(b.0.cmp(&a.0)))?;
    let enough_share = days * 100 >= all_days.len() * HOME_MIN_DAY_SHARE_PERCENT;
    if days < HOME_MIN_DAYS || !enough_share {
        return None;
    }
    Some(HomeCity {
        place: places.get(city).clone(),
        days,
    })
}
