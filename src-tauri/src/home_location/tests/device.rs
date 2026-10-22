use std::cell::Cell;

use super::fixtures::{empty_store, saved_home, store_with_home};
use crate::{
    geo::Coordinates,
    home_location::{
        device::{country_in_table, device_home, DeviceLocator},
        find_home_from_device, use_current_location,
    },
    models::HomeSource,
};

const PERTH: (f64, f64) = (-31.95, 115.86);
const ZONE_TAB: &str = "# comment\nAR\t-3436-05827\tAmerica/Argentina/Buenos_Aires\tBA\n\
AU\t-3352+15113\tAustralia/Sydney\tNew South Wales\n";

#[derive(Default)]
struct FakeLocator {
    point: Option<(f64, f64)>,
    zone: Option<&'static str>,
    asked: Cell<usize>,
}

impl DeviceLocator for FakeLocator {
    fn current_point(&self) -> Option<Coordinates> {
        self.asked.set(self.asked.get() + 1);
        self.point.map(|(lat, lon)| Coordinates::new(lat, lon))
    }

    fn time_zone(&self) -> Option<String> {
        self.zone.map(str::to_owned)
    }
}

fn located(point: (f64, f64)) -> FakeLocator {
    FakeLocator {
        point: Some(point),
        zone: Some("Australia/Sydney"),
        ..FakeLocator::default()
    }
}

fn zoned(zone: &'static str) -> FakeLocator {
    FakeLocator {
        zone: Some(zone),
        ..FakeLocator::default()
    }
}

fn named(locator: &FakeLocator) -> Option<(String, String, HomeSource)> {
    device_home(locator).map(|home| (home.name, home.country, home.source))
}

#[test]
fn a_location_fix_names_the_nearest_city() {
    assert_eq!(
        named(&located(PERTH)),
        Some(("Perth".into(), "Australia".into(), HomeSource::Device))
    );
}

#[test]
fn without_a_fix_the_time_zone_names_home() {
    for (zone, city, country) in [
        ("Australia/Sydney", "Sydney", "Australia"),
        (
            "America/Argentina/Buenos_Aires",
            "Buenos Aires",
            "Argentina",
        ),
    ] {
        assert_eq!(
            named(&zoned(zone)),
            Some((city.into(), country.into(), HomeSource::TimeZone))
        );
    }
    assert_eq!(named(&zoned("Etc/UTC")), None);
    assert_eq!(named(&FakeLocator::default()), None);
}

#[test]
fn zone_tab_gives_the_country_of_a_zone() {
    assert_eq!(country_in_table(ZONE_TAB, "Australia/Sydney"), Some("AU"));
    assert_eq!(country_in_table(ZONE_TAB, "Europe/Paris"), None);
}

#[test]
fn startup_saves_a_device_home_when_none_is_saved() {
    let (_temp, store) = empty_store();

    find_home_from_device(&store, &located(PERTH));

    let home = saved_home(&store).expect("a device home");
    assert_eq!(
        (home.name.as_str(), home.source),
        ("Perth", HomeSource::Device)
    );
}

#[test]
fn startup_replaces_a_time_zone_guess_and_skips_other_homes() {
    let (_temp, guessed) = store_with_home("Sydney", (-33.87, 151.21), HomeSource::TimeZone);
    find_home_from_device(&guessed, &located(PERTH));
    assert_eq!(
        saved_home(&guessed).map(|home| home.name),
        Some("Perth".into())
    );

    let (_temp, learned) = store_with_home("Sydney", (-33.87, 151.21), HomeSource::Learned);
    let locator = located(PERTH);
    find_home_from_device(&learned, &locator);
    assert_eq!(locator.asked.get(), 0);
    assert_eq!(
        saved_home(&learned).map(|home| home.name),
        Some("Sydney".into())
    );
}

#[test]
fn current_location_replaces_a_picked_home() {
    let (_temp, store) = store_with_home("Sydney", (-33.87, 151.21), HomeSource::Manual);

    let settings = use_current_location(&store, &located(PERTH)).expect("saved");

    let home = settings.home.expect("a device home");
    assert_eq!(
        (home.name.as_str(), home.source),
        ("Perth", HomeSource::Device)
    );
}

#[test]
fn current_location_without_a_fix_keeps_home_and_explains() {
    let (_temp, store) = store_with_home("Sydney", (-33.87, 151.21), HomeSource::Manual);

    let error = use_current_location(&store, &zoned("Australia/Sydney")).expect_err("no fix");

    assert!(error.contains("Location Services"), "{error}");
    assert_eq!(
        saved_home(&store).map(|home| home.source),
        Some(HomeSource::Manual)
    );
}
