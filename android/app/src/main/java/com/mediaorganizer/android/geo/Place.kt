package com.mediaorganizer.android.geo

/** A named place. GeoNames names are ASCII. Park names are Latin, or Chinese, Japanese and Korean as written. */
data class Place(
    val name: String,
    val kind: PlaceKind,
    /** State or region, when GeoNames knows it. */
    val admin1: String?,
    val countryCode: String,
    val countryName: String,
    val latitude: Double,
    val longitude: Double,
    val population: Int,
)

/** Every name a point can get, from one search of its surroundings. */
data class Resolution(
    /** What a place folder is named after. */
    val place: Place?,
    /** The city that covers the point, else the nearest city within 50 km. */
    val city: Place?,
)
