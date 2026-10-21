"""Maps GeoNames feature codes to the place kinds the app knows."""

from sources import Row

# (feature class, feature code) -> kind tag. Codes: https://www.geonames.org/export/codes.html
KIND_BY_FEATURE = {
    ("L", "PRK"): "park",
    ("L", "RES"): "reserve",
    ("L", "RESF"): "reserve",
    ("L", "RESN"): "reserve",
    ("L", "RESW"): "reserve",
    ("L", "AMUS"): "amusement_park",
    ("S", "AMUS"): "amusement_park",
    ("S", "GDN"): "garden",
    ("S", "ZOO"): "zoo",
    ("S", "MUS"): "museum",
    ("S", "CSTL"): "castle",
    ("S", "PAL"): "palace",
    ("S", "MNMT"): "monument",
    ("S", "HSTS"): "historic_site",
    ("S", "RUIN"): "ruin",
    ("S", "CH"): "temple",
    ("S", "MSQ"): "temple",
    ("S", "TMPL"): "temple",
    ("S", "MSTY"): "temple",
    ("S", "SHRN"): "temple",
    ("S", "PGDA"): "temple",
    ("T", "BCH"): "beach",
    ("H", "FLLS"): "waterfall",
    ("H", "LK"): "lake",
    ("T", "PK"): "peak",
    ("T", "VLC"): "volcano",
    ("T", "CNYN"): "canyon",
    ("T", "ISL"): "island",
    ("H", "GLCR"): "glacier",
}

# Kept without a Wikipedia link. Visitors name these places and GeoNames lists few small ones.
STRONG_KINDS = {"garden", "zoo", "amusement_park", "national_park"}
STRONG_CODES = {"RESN"}

# GeoNames has no national park code. Large parks are PRK rows with these words in the name.
NATIONAL_PARK_WORDS = ("national park", "nationalpark", "parque nacional", "parc national",
                       "parco nazionale", "park narodowy", "narodni park")
LARGE_PARK_WORDS = ("state park", "provincial park", "regional park", "national forest",
                    "national recreation area", "national monument", "wilderness")
GARDEN_WORDS = ("botanic", "garden", "arboretum")


def kind_of(row: Row) -> str | None:
    kind = KIND_BY_FEATURE.get((row.feature_class, row.feature_code))
    if kind != "park":
        return kind
    name = row.name.lower()
    if any(word in name for word in NATIONAL_PARK_WORDS):
        return "national_park"
    if any(word in name for word in LARGE_PARK_WORDS):
        return "reserve"
    if any(word in name for word in GARDEN_WORDS):
        return "garden"
    return kind


def is_notable(kind: str, row: Row, linked_ids: set[int]) -> bool:
    if kind in STRONG_KINDS or row.feature_code in STRONG_CODES:
        return True
    return row.geoname_id in linked_ids
