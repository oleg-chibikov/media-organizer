package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.Place
import com.mediaorganizer.android.organize.sanitizeForWindowsFilename
import java.nio.file.Path

/**
 * The matched place or city of each geotagged shot, with its country and how much of it is home.
 * Different places that share a name get their region appended, as in `Central Park (New York)`.
 */
internal fun folderPlaces(shots: List<Shot>, places: PlaceTable, home: Home?): MutableMap<Path, FilePlace> {
    class ShotPlace(val path: Path, val place: Int, val city: Int?)
    val shotPlaces = shots.mapNotNull { shot ->
        val resolved = shot.location ?: return@mapNotNull null
        ShotPlace(shot.path, resolved.place ?: resolved.city ?: return@mapNotNull null, resolved.city)
    }
    val folders = shotPlaces.map { it.place }.distinct().associateWithTo(HashMap()) { baseFolder(places[it], home) }
    val sharing = folders.values.groupingBy { it }.eachCount()
    for (entry in folders.entries) {
        if (sharing.getValue(entry.value) > 1) {
            val place = places[entry.key]
            val region = place.admin1 ?: place.countryName
            entry.setValue(entry.value.copy(place = sanitizeForWindowsFilename("${entry.value.place} ($region)")))
        }
    }
    return shotPlaces.associateTo(HashMap()) { shot ->
        val folder = folders.getValue(shot.place)
        val inHomeCity = shot.city != null && isHomeCity(places[shot.city], home)
        shot.path to if (inHomeCity) folder.copy(home = HomeMatch.CITY) else folder
    }
}

private fun baseFolder(place: Place, home: Home?): FilePlace {
    val inHomeCountry = home != null && home.country.equals(place.countryName, ignoreCase = true)
    return FilePlace(
        place = sanitizeForWindowsFilename(place.name),
        country = sanitizeForWindowsFilename(place.countryName),
        home = if (inHomeCountry) HomeMatch.COUNTRY else HomeMatch.AWAY,
    )
}

private fun isHomeCity(city: Place, home: Home?): Boolean = home != null &&
    home.city.equals(city.name, ignoreCase = true) &&
    home.country.equals(city.countryName, ignoreCase = true)
