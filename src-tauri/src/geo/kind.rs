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
    const ALL: [Self; 22] = [
        Self::City,
        Self::Park,
        Self::NationalPark,
        Self::Reserve,
        Self::Garden,
        Self::Zoo,
        Self::AmusementPark,
        Self::Museum,
        Self::Castle,
        Self::Palace,
        Self::Monument,
        Self::HistoricSite,
        Self::Ruin,
        Self::Temple,
        Self::Beach,
        Self::Waterfall,
        Self::Lake,
        Self::Peak,
        Self::Volcano,
        Self::Canyon,
        Self::Island,
        Self::Glacier,
    ];

    /// The tag `scripts/geodata/kinds.py` writes, also sent to the frontend.
    pub(crate) fn tag(self) -> &'static str {
        match self {
            Self::City => "city",
            Self::Park => "park",
            Self::NationalPark => "national_park",
            Self::Reserve => "reserve",
            Self::Garden => "garden",
            Self::Zoo => "zoo",
            Self::AmusementPark => "amusement_park",
            Self::Museum => "museum",
            Self::Castle => "castle",
            Self::Palace => "palace",
            Self::Monument => "monument",
            Self::HistoricSite => "historic_site",
            Self::Ruin => "ruin",
            Self::Temple => "temple",
            Self::Beach => "beach",
            Self::Waterfall => "waterfall",
            Self::Lake => "lake",
            Self::Peak => "peak",
            Self::Volcano => "volcano",
            Self::Canyon => "canyon",
            Self::Island => "island",
            Self::Glacier => "glacier",
        }
    }

    pub(super) fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.tag() == tag)
    }
}
