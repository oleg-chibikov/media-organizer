package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.geo.Place
import com.mediaorganizer.android.geo.Resolution
import kotlin.math.roundToLong

/** The resolved place of a point and the city it sits in, as ids of a [PlaceTable]. */
internal data class Resolved(val place: Int?, val city: Int?)

/** Geocodes GPS points once and gives each place a small id. */
internal class PlaceTable(private val geocode: (Coordinates) -> Resolution) {
    private val places = mutableListOf<Place>()
    private val ids = HashMap<Triple<String, Double, Double>, Int>()
    // Burst photos share a point, so nearby points reuse one lookup.
    private val cache = HashMap<Pair<Long, Long>, Resolved>()

    fun resolve(point: Coordinates): Resolved {
        val key = cacheStep(point.latitude) to cacheStep(point.longitude)
        return cache.getOrPut(key) {
            val resolution = geocode(point)
            Resolved(resolution.place?.let(::intern), resolution.city?.let(::intern))
        }
    }

    operator fun get(id: Int): Place = places[id]

    private fun intern(place: Place): Int = ids.getOrPut(Triple(place.name, place.latitude, place.longitude)) {
        places += place
        places.size - 1
    }
}

private fun cacheStep(degrees: Double): Long = (degrees / GEOCODE_CACHE_STEP_DEGREES).roundToLong()
