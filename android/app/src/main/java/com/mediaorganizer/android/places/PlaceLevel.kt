package com.mediaorganizer.android.places

import java.nio.file.Path
import java.time.LocalDate
import java.util.Locale
import java.util.SortedSet

/** The date folder a country or place folder sits in. Higher levels come first. */
enum class PlaceLevel {
    /** Files in more than one month: `<year>/<folder>/<month>/<day>`. */
    YEAR,
    /** Files on more than one day of one month: `<year>/<month>/<folder>/<day>`. */
    MONTH,
    /** Files on one day: `<year>/<month>/<day>/<folder>`. */
    DAY,
}

/** The days a folder's files were taken on, within one year. */
internal class DaySpan {
    // `month * 100 + day`, so the set sorts by date.
    private val days = sortedSetOf<Int>()

    fun add(date: LocalDate) {
        days += date.monthValue * 100 + date.dayOfMonth
    }

    fun months(): SortedSet<Int> = days.mapTo(sortedSetOf()) { it / 100 }

    fun level(): PlaceLevel = when {
        months().size > 1 -> PlaceLevel.YEAR
        days.size > 1 -> PlaceLevel.MONTH
        else -> PlaceLevel.DAY
    }

    fun firstDate(year: Int): LocalDate? = days.firstOrNull()?.let { LocalDate.of(year, it / 100, it % 100) }
}

/**
 * `<year>/<month>/<day>` with each folder after the date part of its level, relative to the root.
 * With [toLastFolder] the path stops at the deepest folder.
 */
internal fun datedDir(date: LocalDate, folders: List<Pair<String, PlaceLevel>>, toLastFolder: Boolean): Path {
    val deepest = folders.maxOfOrNull { it.second }
    val dateParts = listOf(String.format(Locale.ROOT, "%04d", date.year), "${date.monthValue}", "${date.dayOfMonth}")
    val parts = mutableListOf<String>()
    for ((level, part) in PlaceLevel.entries.zip(dateParts)) {
        parts += part
        parts += folders.filter { it.second == level }.map { it.first }
        if (toLastFolder && level == deepest) break
    }
    return Path.of(parts.first(), *parts.drop(1).toTypedArray())
}
