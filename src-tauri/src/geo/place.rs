use super::kind::PlaceKind;

/// A named place from the GeoNames extract. Names are ASCII.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Place {
    pub(crate) name: String,
    pub(crate) kind: PlaceKind,
    /// State or region, when GeoNames knows it.
    pub(crate) admin1: Option<String>,
    pub(crate) country_code: String,
    pub(crate) country_name: String,
    pub(crate) latitude: f64,
    pub(crate) longitude: f64,
    pub(crate) population: u32,
}
