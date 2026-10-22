package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.geo.Geocoder
import com.mediaorganizer.android.geo.Place

private const val CANDIDATES = 50

/** Where the device is. The app asks Android, tests pass a fake. */
interface DeviceLocator {
    /** One location fix, or null when denied, unavailable or too slow. */
    fun currentPoint(): Coordinates?

    /** The IANA zone name, such as `Australia/Sydney`. */
    fun timeZone(): String?

    /** The country code of [zone], such as `AU`, when known. */
    fun countryOfZone(zone: String): String?
}

/** Saves a home from the device when none is saved, or only a time zone guess is, as `find_home_from_device`. */
fun findHomeFromDevice(homes: HomeStore, locator: DeviceLocator, geocoder: Geocoder) {
    val saved = homes.saved()
    if (saved != null && saved.source != HomeSource.TIME_ZONE) return
    deviceHome(locator, geocoder)?.let(homes::offer)
}

/** The city at the device's location, else the city its time zone names. */
fun deviceHome(locator: DeviceLocator, geocoder: Geocoder): HomeLocation? =
    locatedHome(locator, geocoder) ?: timeZoneHome(locator, geocoder)

/** The city nearest the device's location fix, else the nearest place. */
fun locatedHome(locator: DeviceLocator, geocoder: Geocoder): HomeLocation? {
    val resolution = geocoder.resolve(locator.currentPoint() ?: return null)
    return (resolution.city ?: resolution.place)?.let { homeAt(it, HomeSource.DEVICE) }
}

private fun timeZoneHome(locator: DeviceLocator, geocoder: Geocoder): HomeLocation? {
    val zone = locator.timeZone() ?: return null
    val city = cityForZone(zone, locator.countryOfZone(zone), geocoder::searchCities) ?: return null
    return homeAt(city, HomeSource.TIME_ZONE)
}

/**
 * The biggest city named like the zone's last part, such as Sydney for `Australia/Sydney`, in
 * [countryCode] when it is known. Zones such as `Etc/UTC` name no city.
 */
internal fun cityForZone(zone: String, countryCode: String?, searchCities: (String, Int) -> List<Place>): Place? {
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
