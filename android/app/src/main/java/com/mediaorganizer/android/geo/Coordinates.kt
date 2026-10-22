package com.mediaorganizer.android.geo

import kotlin.math.asin
import kotlin.math.cos
import kotlin.math.min
import kotlin.math.pow
import kotlin.math.sin
import kotlin.math.sqrt

private const val EARTH_RADIUS_KM = 6371.0088

/** A point in decimal degrees. */
data class Coordinates(val latitude: Double, val longitude: Double)

/** Haversine distance in kilometres. */
fun distanceKm(a: Coordinates, b: Coordinates): Double {
    val latA = Math.toRadians(a.latitude)
    val latB = Math.toRadians(b.latitude)
    val halfDLat = (latB - latA) / 2.0
    val halfDLon = Math.toRadians(b.longitude - a.longitude) / 2.0
    val h = sin(halfDLat).pow(2) + cos(latA) * cos(latB) * sin(halfDLon).pow(2)
    return 2.0 * EARTH_RADIUS_KM * asin(min(sqrt(h), 1.0))
}
