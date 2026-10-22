use crate::{
    models::{HomeLocation, HomeSource},
    settings::{learn_home_location, set_home_location, storage::default_destination_settings},
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
    let picked = set_home_location(
        &default_destination_settings(),
        Some(home("Sydney", HomeSource::Manual)),
    );
    let cleared = set_home_location(&picked, None);
    let learned = learn_home_location(&cleared, home("Perth", HomeSource::Manual));

    assert_eq!(learned.home, Some(home("Perth", HomeSource::Learned)));
}

#[test]
fn learning_keeps_a_saved_home() {
    let defaults = default_destination_settings();
    let picked = set_home_location(&defaults, Some(home("Sydney", HomeSource::Manual)));
    let learned = learn_home_location(&defaults, home("Sydney", HomeSource::Learned));

    for saved in [picked, learned] {
        let relearned = learn_home_location(&saved, home("Perth", HomeSource::Learned));

        assert_eq!(relearned.home, saved.home);
    }
}
