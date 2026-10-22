package com.mediaorganizer.android.geo

import kotlin.math.abs
import kotlin.math.cos
import kotlin.math.floor
import kotlin.math.max
import kotlin.math.min

private const val LAT_CELLS = 180
internal const val LON_CELLS = 360
internal const val CELL_COUNT = LAT_CELLS * LON_CELLS
internal const val KM_PER_DEGREE = 111.2
// Above this latitude a search circle spans every longitude.
private const val POLAR_LATITUDE = 89.0

/** The one-degree cell of a point. */
internal fun cellOf(latitude: Double, longitude: Double): Int {
    val row = floor(latitude + 90.0).toInt().coerceIn(0, LAT_CELLS - 1)
    val column = Math.floorMod(floor(longitude + 180.0).toInt(), LON_CELLS)
    return row * LON_CELLS + column
}

/** A one-degree grid over the globe. Entries sorted by cell let a lookup scan only the cells a circle touches. */
internal class Grid private constructor(
    /** Entry range of cell `c` is `starts[c] until starts[c + 1]`. */
    private val starts: IntArray,
) {
    /** Calls [action] with the entry range of each cell within [radiusKm] of [point]. */
    fun forEachRangeNear(point: Coordinates, radiusKm: Double, action: (Int, Int) -> Unit) {
        val span = radiusKm / KM_PER_DEGREE
        val south = max(point.latitude - span, -90.0)
        val north = min(point.latitude + span, 90.0)
        val widest = max(abs(south), abs(north))
        val lonSpan = span / cos(Math.toRadians(widest))
        val everyLongitude = widest >= POLAR_LATITUDE || lonSpan >= 179.0
        val west = if (everyLongitude) -180.0 else point.longitude - lonSpan
        val east = if (everyLongitude) 179.0 else point.longitude + lonSpan
        for (row in rowOf(south)..rowOf(north)) {
            for (column in columnIndex(west)..columnIndex(east)) {
                val cell = row * LON_CELLS + Math.floorMod(column, LON_CELLS)
                action(starts[cell], starts[cell + 1])
            }
        }
    }

    companion object {
        /** Counts entries per cell. [cells] holds the cell of every entry, in sorted order. */
        fun ofSortedCells(cells: IntArray): Grid {
            val starts = IntArray(CELL_COUNT + 1)
            for (cell in cells) starts[cell + 1]++
            for (cell in 1 until starts.size) starts[cell] += starts[cell - 1]
            return Grid(starts)
        }
    }
}

private fun rowOf(latitude: Double): Int = min(floor(latitude + 90.0).toInt(), LAT_CELLS - 1)

private fun columnIndex(longitude: Double): Int = floor(longitude + 180.0).toInt()
