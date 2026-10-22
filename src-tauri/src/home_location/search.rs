use crate::{geo, models::HomeCandidate};

const MAX_CANDIDATES: usize = 10;

/// Cities whose name starts with, then contains, `query`, biggest first.
pub(crate) fn search_home_cities(query: &str) -> Vec<HomeCandidate> {
    geo::search_cities(query, MAX_CANDIDATES)
        .into_iter()
        .map(|city| HomeCandidate {
            name: city.name,
            region: city.admin1,
            country: city.country_name,
            latitude: city.latitude,
            longitude: city.longitude,
        })
        .collect()
}
