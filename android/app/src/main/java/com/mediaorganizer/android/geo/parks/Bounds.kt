package com.mediaorganizer.android.geo.parks

import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.geo.KM_PER_DEGREE
import kotlin.math.cos
import kotlin.math.max

/** A latitude and longitude box that lets a lookup skip far parks and edges. */
internal data class Bounds(val south: Float, val west: Float, val north: Float, val east: Float) {
    fun widened(degrees: Float) = Bounds(south - degrees, west - degrees, north + degrees, east + degrees)

    fun union(other: Bounds) = Bounds(
        minOf(south, other.south),
        minOf(west, other.west),
        maxOf(north, other.north),
        maxOf(east, other.east),
    )

    fun covers(point: Coordinates): Boolean {
        val latitude = point.latitude.toFloat()
        val longitude = point.longitude.toFloat()
        return latitude in south..north && longitude in west..east
    }

    /** Whether a ray going east from [point] can cross an edge inside the box. */
    fun meetsEastwardRay(point: Coordinates): Boolean {
        val latitude = point.latitude.toFloat()
        return latitude in south..north && point.longitude.toFloat() <= east
    }

    companion object {
        /** The box around points `from until to` of interleaved `lat, lon` [points]. Null for no points. */
        fun of(points: FloatArray, from: Int = 0, to: Int = points.size / 2): Bounds? {
            if (from >= to) return null
            var south = points[from * 2]
            var north = south
            var west = points[from * 2 + 1]
            var east = west
            for (index in from + 1 until to) {
                val latitude = points[index * 2]
                val longitude = points[index * 2 + 1]
                south = minOf(south, latitude)
                north = maxOf(north, latitude)
                west = minOf(west, longitude)
                east = maxOf(east, longitude)
            }
            return Bounds(south, west, north, east)
        }
    }
}

/** Degrees that span at least [km] around [point] both ways. Longitude degrees shrink toward the poles. */
internal fun marginDegrees(point: Coordinates, km: Double): Float {
    val cos = max(cos(Math.toRadians(point.latitude)), 0.01)
    return (km / (KM_PER_DEGREE * cos)).toFloat()
}
