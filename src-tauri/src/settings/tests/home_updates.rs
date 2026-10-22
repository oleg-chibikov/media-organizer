use crate::{
    models::{DestinationSettings, HomeLocation, HomeSource},
    settings::{
        offer_home_location, replace_home_location, set_home_location,
        storage::default_destination_settings,
    },
};

fn home(name: &str, source: HomeSource) -> HomeLocation {
    HomeLocation {
        name: name.to_owned(),
        country: "Australia".to_owned(),
        latitude: -33.87,
        longitude: 151.21,
        source,
    }
}

fn saved(source: HomeSource) -> DestinationSettings {
    replace_home_location(&default_destination_settings(), home("Sydney", source))
}

#[test]
fn a_picked_home_is_saved_as_manual() {
    let settings = set_home_location(
        &default_destination_settings(),
        Some(home("Sydney", HomeSource::Learned)),
    );

    assert_eq!(settings.home, Some(home("Sydney", HomeSource::Manual)));
}

#[test]
fn clearing_home_lets_a_scan_learn_it_again() {
    let cleared = set_home_location(&saved(HomeSource::Manual), None);
    let learned = offer_home_location(&cleared, home("Perth", HomeSource::Learned));

    assert_eq!(learned.home, Some(home("Perth", HomeSource::Learned)));
}

#[test]
fn a_stronger_source_replaces_a_weaker_one() {
    let pairs = [
        (HomeSource::TimeZone, HomeSource::Learned),
        (HomeSource::TimeZone, HomeSource::Device),
        (HomeSource::Learned, HomeSource::Device),
    ];
    for (weaker, stronger) in pairs {
        let offered = offer_home_location(&saved(weaker), home("Perth", stronger));

        assert_eq!(offered.home, Some(home("Perth", stronger)), "{weaker:?}");
    }
}

#[test]
fn a_weaker_or_equal_source_keeps_the_saved_home() {
    let pairs = [
        (HomeSource::Manual, HomeSource::Device),
        (HomeSource::Device, HomeSource::Learned),
        (HomeSource::Learned, HomeSource::Learned),
        (HomeSource::Learned, HomeSource::TimeZone),
    ];
    for (stronger, weaker) in pairs {
        let settings = saved(stronger);

        let offered = offer_home_location(&settings, home("Perth", weaker));

        assert_eq!(offered.home, settings.home, "{stronger:?}");
    }
}

#[test]
fn a_requested_lookup_replaces_a_manual_home() {
    let replaced = replace_home_location(
        &saved(HomeSource::Manual),
        home("Perth", HomeSource::Device),
    );

    assert_eq!(replaced.home, Some(home("Perth", HomeSource::Device)));
}
