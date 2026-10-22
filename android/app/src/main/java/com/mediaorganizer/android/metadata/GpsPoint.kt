package com.mediaorganizer.android.metadata

import com.mediaorganizer.android.geo.Coordinates

/** Drops values outside the globe and the (0, 0) that cameras write without a fix, as desktop does. */
internal fun gpsPoint(latitude: Double, longitude: Double): Coordinates? {
    val inRange = latitude in -90.0..90.0 && longitude in -180.0..180.0
    val isNullIsland = latitude == 0.0 && longitude == 0.0
    return if (inRange && !isNullIsland) Coordinates(latitude, longitude) else null
}

/** Reads the ISO 6709 form videos carry, such as `+37.7749-122.4194+010.000/`. */
internal fun parseIso6709(text: String?): Coordinates? {
    val body = text?.trim()?.trimEnd('/') ?: return null
    val starts = body.indices.filter { body[it] == '+' || body[it] == '-' }
    if (starts.firstOrNull() != 0 || starts.size < 2) return null
    val endOfLongitude = starts.getOrNull(2) ?: body.length
    val latitude = body.substring(starts[0], starts[1]).toDoubleOrNull() ?: return null
    val longitude = body.substring(starts[1], endOfLongitude).toDoubleOrNull() ?: return null
    return gpsPoint(latitude, longitude)
}
