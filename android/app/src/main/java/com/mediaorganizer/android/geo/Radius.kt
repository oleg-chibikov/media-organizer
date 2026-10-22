package com.mediaorganizer.android.geo

import kotlin.math.sqrt

// GeoNames gives one point per place and no outline, so each kind gets a radius that roughly
// covers its usual size. Same numbers as `src-tauri/src/geo/radius.rs`.

internal const val CITY_RADIUS_KM = 50.0
internal const val FALLBACK_RADIUS_KM = 150.0

internal const val SMALL_SITE_RADIUS_KM = 1.0
internal const val CITY_PARK_RADIUS_KM = 1.5
// Being near a summit means standing on it. Valleys below are often closer than 3 km.
internal const val SUMMIT_RADIUS_KM = 1.0
internal const val NATURE_SPOT_RADIUS_KM = 3.0
internal const val LANDFORM_RADIUS_KM = 5.0
internal const val ISLAND_RADIUS_KM = 10.0
internal const val RESERVE_RADIUS_KM = 15.0

// Gives San Francisco about 18 km and a town of 5,000 about 1.4 km.
private const val CITY_REACH_KM_PER_SQRT_PERSON = 0.02

/** How far a city spreads from its point, growing with population, up to [CITY_RADIUS_KM]. */
internal fun cityReachKm(population: Int): Double =
    minOf(CITY_REACH_KM_PER_SQRT_PERSON * sqrt(population.toDouble()), CITY_RADIUS_KM)
