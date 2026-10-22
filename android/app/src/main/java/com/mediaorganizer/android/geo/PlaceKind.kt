package com.mediaorganizer.android.geo

/**
 * What kind of place a GeoNames row describes. [tag] is what `scripts/geodata/kinds.py` writes.
 * [matchRadiusKm] is how far from its point a visit still counts as inside: null for cities, the
 * fallback matched by distance alone, and for national parks, whose outlines come from `parks`.
 * Lower [nestingLevel]s sit inside higher ones and win when both match.
 */
enum class PlaceKind(val tag: String, internal val matchRadiusKm: Double?, internal val nestingLevel: Int) {
    CITY("city", null, 6),
    PARK("park", CITY_PARK_RADIUS_KM, 1),
    NATIONAL_PARK("national_park", null, 5),
    RESERVE("reserve", RESERVE_RADIUS_KM, 4),
    GARDEN("garden", SMALL_SITE_RADIUS_KM, 0),
    ZOO("zoo", SMALL_SITE_RADIUS_KM, 0),
    AMUSEMENT_PARK("amusement_park", CITY_PARK_RADIUS_KM, 1),
    MUSEUM("museum", SMALL_SITE_RADIUS_KM, 0),
    CASTLE("castle", SMALL_SITE_RADIUS_KM, 0),
    PALACE("palace", SMALL_SITE_RADIUS_KM, 0),
    MONUMENT("monument", SMALL_SITE_RADIUS_KM, 0),
    HISTORIC_SITE("historic_site", SMALL_SITE_RADIUS_KM, 0),
    RUIN("ruin", SMALL_SITE_RADIUS_KM, 0),
    TEMPLE("temple", SMALL_SITE_RADIUS_KM, 0),
    BEACH("beach", NATURE_SPOT_RADIUS_KM, 2),
    WATERFALL("waterfall", NATURE_SPOT_RADIUS_KM, 2),
    LAKE("lake", NATURE_SPOT_RADIUS_KM, 2),
    PEAK("peak", SUMMIT_RADIUS_KM, 0),
    VOLCANO("volcano", LANDFORM_RADIUS_KM, 3),
    CANYON("canyon", LANDFORM_RADIUS_KM, 3),
    // An island can hold any other place, so it nests above what its radius says.
    ISLAND("island", ISLAND_RADIUS_KM, 6),
    GLACIER("glacier", LANDFORM_RADIUS_KM, 3),
    ;

    companion object {
        private val byTag = entries.associateBy(PlaceKind::tag)

        fun fromTag(tag: String): PlaceKind? = byTag[tag]
    }
}
