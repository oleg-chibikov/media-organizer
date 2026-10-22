package com.mediaorganizer.android.geo

import com.mediaorganizer.android.geo.parks.Parks
import com.mediaorganizer.android.geo.parks.parseParks
import java.io.InputStream

/** Opens a place table by name, `geodata.tsv` or `parks.tsv`, plain or gzipped. */
fun interface AssetOpener {
    fun open(name: String): InputStream
}

/**
 * Offline reverse geocoding over the desktop app's GeoNames extract (CC BY 4.0) and OpenStreetMap
 * park outlines (ODbL). Each table loads on first use, so call lookups off the main thread.
 */
class Geocoder(private val assets: AssetOpener) {
    private val index by lazy { assets.open("geodata.tsv").use(::parseGeoIndex) }
    private val parks: Parks by lazy { assets.open("parks.tsv").use(::parseParks) }
    private val cityNames by lazy { CityNames(index) }

    fun resolve(point: Coordinates): Resolution {
        val nearby = Nearby(index, point)
        return Resolution(place = placeIn(nearby), city = cityIn(nearby))
    }

    /** Up to [limit] cities whose name starts with [query], then ones that contain it, biggest first. */
    fun searchCities(query: String, limit: Int): List<Place> = cityNames.search(query, limit)

    /**
     * The park or city that covers the point, else the point of interest that contains it, else the
     * nearest city within 50 km, else the nearest place of any kind within 150 km.
     */
    private fun placeIn(nearby: Nearby): Place? {
        parkOrCityPlace(nearby, parks)?.let { return it }
        val entry = nearby.matchingPoi()
            ?: nearby.nearestCity()
            ?: index.nearest(nearby.point, FALLBACK_RADIUS_KM)
            ?: return null
        return index.place(entry)
    }

    /** The city that covers the point, else the nearest city within 50 km. */
    private fun cityIn(nearby: Nearby): Place? = (coveringCity(nearby) ?: nearby.nearestCity())?.let(index::place)
}
