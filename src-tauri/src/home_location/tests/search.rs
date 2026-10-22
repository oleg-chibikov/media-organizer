use crate::home_location::search_home_cities;

#[test]
fn home_search_returns_up_to_ten_candidates_with_regions() {
    let found = search_home_cities("zür");

    assert_eq!(found[0].name, "Zuerich");
    assert_eq!(found[0].country, "Switzerland");
    assert!(found[0].region.is_some());
    assert_eq!(search_home_cities("a").len(), 10);
}
