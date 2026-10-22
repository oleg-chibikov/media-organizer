package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.Place

private const val CANDIDATES = 50

/**
 * The biggest city named like the zone's last part, such as Sydney for `Australia/Sydney`, in
 * [countryCode] when it is known. Zones such as `Etc/UTC` name no city.
 */
fun cityForZone(zone: String, countryCode: String?, searchCities: (String, Int) -> List<Place>): Place? {
    val slash = zone.lastIndexOf('/')
    if (slash < 0 || zone.startsWith("Etc")) return null
    val name = zone.substring(slash + 1).replace('_', ' ')
    val candidates = searchCities(name, CANDIDATES)
    // Starts-with matches come first, so `Ho Chi Minh` finds `Ho Chi Minh City`.
    return if (countryCode != null) {
        candidates.firstOrNull { it.countryCode.equals(countryCode, ignoreCase = true) }
    } else {
        candidates.firstOrNull { it.name.equals(name, ignoreCase = true) }
    }
}
