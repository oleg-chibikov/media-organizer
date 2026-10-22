package com.mediaorganizer.android.geo.parks

import com.mediaorganizer.android.geo.CELL_COUNT
import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.geo.LON_CELLS
import com.mediaorganizer.android.geo.PlaceKind
import com.mediaorganizer.android.geo.cellOf

/** Parks join the cells this close to their bounds too, so [Parks.near] finds a park across a cell edge in one cell. */
private const val NEAR_MARGIN_DEGREES = 0.1f
/** Below [NEAR_MARGIN_DEGREES] at latitudes people photograph. */
private const val MAX_NEAR_KM = 5.0

internal class Park(
    val name: String,
    val kind: PlaceKind,
    val areaKm2: Float,
    /** A point inside the park, used to find its country. */
    val label: Coordinates,
    val outline: Outline,
)

/** Park outlines from OpenStreetMap, through Overture Maps (ODbL), indexed by one-degree cell. */
internal class Parks(private val parks: List<Park>) {
    /** The parks whose bounds touch each cell. */
    private val cells = arrayOfNulls<IntArray>(CELL_COUNT)

    init {
        val lists = HashMap<Int, MutableList<Int>>()
        parks.forEachIndexed { id, park ->
            val bounds = park.outline.bounds.widened(NEAR_MARGIN_DEGREES)
            val first = corner(bounds.south, bounds.west)
            val last = corner(bounds.north, bounds.east)
            for (row in first / LON_CELLS..last / LON_CELLS) {
                for (column in first % LON_CELLS..last % LON_CELLS) {
                    lists.getOrPut(row * LON_CELLS + column) { mutableListOf() } += id
                }
            }
        }
        for ((cell, ids) in lists) cells[cell] = ids.toIntArray()
    }

    val size: Int get() = parks.size

    /** The smallest park that contains [point]. */
    fun containing(point: Coordinates): Park? =
        inCell(point).filter { it.outline.contains(point) }.minByOrNull(Park::areaKm2)

    /** Parks within [radiusKm] of [point], up to 5 km, with their distance. */
    fun near(point: Coordinates, radiusKm: Double): List<Pair<Park, Double>> {
        val radius = minOf(radiusKm, MAX_NEAR_KM)
        val margin = marginDegrees(point, radius)
        return inCell(point)
            .filter { it.outline.bounds.widened(margin).covers(point) }
            .map { park ->
                val distance = if (park.outline.contains(point)) 0.0 else park.outline.distanceKm(point, radius)
                park to distance
            }
            .filter { (_, distance) -> distance <= radius }
    }

    private fun inCell(point: Coordinates): List<Park> =
        cells[cellOf(point.latitude, point.longitude)]?.map { parks[it] }.orEmpty()
}

private fun corner(latitude: Float, longitude: Float): Int =
    cellOf(latitude.toDouble(), longitude.coerceIn(-180f, 179.99f).toDouble())
