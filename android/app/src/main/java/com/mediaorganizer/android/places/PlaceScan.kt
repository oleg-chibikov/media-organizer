package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.geo.Resolution
import java.nio.file.Path

/** One run's files geocoded once, ready for home inference and place folders. */
class PlaceScan(files: List<PlaceInput>, geocode: (Coordinates) -> Resolution) {
    private val places = PlaceTable(geocode)
    private val shots = collectShots(files, places)

    /** The city with the most days, when it has enough of them to be home. */
    internal fun homeCity(): HomeCity? = inferHome(shots, places)

    /** The place of each geotagged file, and of each file with no GPS taken by the same device close in time. */
    fun detect(home: Home?): Map<Path, FilePlace> {
        val filePlaces = folderPlaces(shots, places, home)
        filePlaces += untaggedPlaces(shots, filePlaces)
        return filePlaces
    }
}
