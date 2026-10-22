use super::{Media, CASOLE_D_ELSA, FRECHEN, HOFSTETTEN, LIVERPOOL_NSW, REGGIO_EMILIA};
use crate::{
    geo::{distance_km, Coordinates},
    trips::{home::infer_home, place_table::PlaceTable, shots::collect_shots},
};

/// Adds one file a day for `days` days from `first_day` at `point`.
fn daily(media: &mut Media, prefix: &str, first_day: &str, days: usize, point: (f64, f64)) {
    media.burst(
        prefix,
        &format!("{first_day} 12:00:00"),
        days,
        24 * 60,
        Some(point),
    );
}

fn home_of(media: &Media) -> Option<Coordinates> {
    let mut places = PlaceTable::default();
    let shots = collect_shots(&media.files, &media.metadata, &mut places);
    infer_home(&shots, &places)
}

#[test]
fn the_city_with_most_days_is_home() {
    let mut media = Media::default();
    daily(&mut media, "home", "2025-01-01", 20, LIVERPOOL_NSW);
    daily(&mut media, "away", "2025-03-01", 5, FRECHEN);

    let home = home_of(&media).expect("a home");

    let liverpool = Coordinates::new(LIVERPOOL_NSW.0, LIVERPOOL_NSW.1);
    assert!(distance_km(home, liverpool) < 10.0, "{home:?}");
}

#[test]
fn too_few_days_give_no_home() {
    let mut media = Media::default();
    daily(&mut media, "home", "2025-01-01", 13, LIVERPOOL_NSW);

    assert_eq!(home_of(&media), None);
}

#[test]
fn a_small_share_of_days_gives_no_home() {
    let mut media = Media::default();
    daily(&mut media, "home", "2025-01-01", 14, LIVERPOOL_NSW);
    daily(&mut media, "a", "2025-02-01", 10, FRECHEN);
    daily(&mut media, "b", "2025-03-01", 10, HOFSTETTEN);
    daily(&mut media, "c", "2025-04-01", 10, CASOLE_D_ELSA);
    daily(&mut media, "d", "2025-05-01", 10, REGGIO_EMILIA);

    assert_eq!(home_of(&media), None);
}
