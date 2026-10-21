//! What kind of place a GeoNames row describes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PlaceKind {
    City,
    Park,
    NationalPark,
    Reserve,
    Garden,
    Zoo,
    AmusementPark,
    Museum,
    Castle,
    Palace,
    Monument,
    HistoricSite,
    Ruin,
    Temple,
    Beach,
    Waterfall,
    Lake,
    Peak,
    Volcano,
    Canyon,
    Island,
    Glacier,
}

impl PlaceKind {
    /// Reads the kind tag that `scripts/geodata/kinds.py` writes.
    pub(super) fn from_tag(tag: &str) -> Option<Self> {
        Some(match tag {
            "city" => Self::City,
            "park" => Self::Park,
            "national_park" => Self::NationalPark,
            "reserve" => Self::Reserve,
            "garden" => Self::Garden,
            "zoo" => Self::Zoo,
            "amusement_park" => Self::AmusementPark,
            "museum" => Self::Museum,
            "castle" => Self::Castle,
            "palace" => Self::Palace,
            "monument" => Self::Monument,
            "historic_site" => Self::HistoricSite,
            "ruin" => Self::Ruin,
            "temple" => Self::Temple,
            "beach" => Self::Beach,
            "waterfall" => Self::Waterfall,
            "lake" => Self::Lake,
            "peak" => Self::Peak,
            "volcano" => Self::Volcano,
            "canyon" => Self::Canyon,
            "island" => Self::Island,
            "glacier" => Self::Glacier,
            _ => return None,
        })
    }
}
