package com.mediaorganizer.android.geo.parks

import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.geo.KM_PER_DEGREE
import kotlin.math.cos
import kotlin.math.hypot
import kotlin.math.min

// Each run of edges keeps its own box, so a test skips most of a long ring.
private const val CHUNK_EDGES = 64

/** A park's outer rings and holes, and whether a point sits inside or near them. */
internal class Outline private constructor(private val rings: List<Ring>, val bounds: Bounds) {
    /** An even-odd ray test. A point in a hole crosses an even number of edges. */
    fun contains(point: Coordinates): Boolean {
        if (!bounds.covers(point)) return false
        val y = point.latitude
        val x = point.longitude
        var crossings = 0
        forEachEdgeWhere({ it.meetsEastwardRay(point) }) { aY, aX, bY, bX ->
            if ((aY > y) != (bY > y) && x < (bX - aX) * (y - aY) / (bY - aY) + aX) crossings++
        }
        return crossings % 2 == 1
    }

    /**
     * Distance to the nearest edge on a flat map, close enough over a few km. Edges farther than
     * [radiusKm] are skipped, so a farther result is infinite.
     */
    fun distanceKm(point: Coordinates, radiusKm: Double): Double {
        val margin = marginDegrees(point, radiusKm)
        val xScale = cos(Math.toRadians(point.latitude))
        var nearest = Double.POSITIVE_INFINITY
        forEachEdgeWhere({ it.widened(margin).covers(point) }) { aY, aX, bY, bX ->
            val distance = distanceToSegment(
                (aY - point.latitude) * KM_PER_DEGREE,
                (aX - point.longitude) * KM_PER_DEGREE * xScale,
                (bY - point.latitude) * KM_PER_DEGREE,
                (bX - point.longitude) * KM_PER_DEGREE * xScale,
            )
            nearest = min(nearest, distance)
        }
        return nearest
    }

    /** Calls [action] with `aLat, aLon, bLat, bLon` of each edge in a chunk [wanted] accepts. */
    private inline fun forEachEdgeWhere(
        wanted: (Bounds) -> Boolean,
        action: (Double, Double, Double, Double) -> Unit,
    ) {
        for (ring in rings) {
            val points = ring.points
            val count = points.size / 2
            ring.chunks.forEachIndexed { index, chunk ->
                if (!wanted(chunk)) return@forEachIndexed
                for (edge in index * CHUNK_EDGES until min((index + 1) * CHUNK_EDGES, count)) {
                    val next = (edge + 1) % count
                    action(
                        points[edge * 2].toDouble(),
                        points[edge * 2 + 1].toDouble(),
                        points[next * 2].toDouble(),
                        points[next * 2 + 1].toDouble(),
                    )
                }
            }
        }
    }

    private class Ring(
        /** Interleaved `lat, lon` points. The last joins back to the first. */
        val points: FloatArray,
    ) {
        /** The box of each run of [CHUNK_EDGES] edges. */
        val chunks: List<Bounds> = run {
            val count = points.size / 2
            (0 until (count + CHUNK_EDGES - 1) / CHUNK_EDGES).map { index ->
                val first = index * CHUNK_EDGES
                val last = min(first + CHUNK_EDGES, count)
                // The last edge of the chunk ends at the first point of the next one.
                Bounds.of(points, first, last)!!.union(Bounds.of(points, last % count, last % count + 1)!!)
            }
        }
    }

    companion object {
        fun of(rings: List<FloatArray>): Outline? {
            val filled = rings.filter { it.size >= 2 }
            val bounds = filled.mapNotNull { Bounds.of(it) }.reduceOrNull(Bounds::union) ?: return null
            return Outline(filled.map(::Ring), bounds)
        }
    }
}

/** Distance from the origin to the segment from `a` to `b`. */
private fun distanceToSegment(aX: Double, aY: Double, bX: Double, bY: Double): Double {
    val dx = bX - aX
    val dy = bY - aY
    val lengthSquared = dx * dx + dy * dy
    val t = if (lengthSquared == 0.0) 0.0 else (-(aX * dx + aY * dy) / lengthSquared).coerceIn(0.0, 1.0)
    return hypot(aX + t * dx, aY + t * dy)
}
