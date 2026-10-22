package com.mediaorganizer.android.places

import java.io.IOException
import java.nio.file.Path
import java.time.LocalDate
import kotlin.io.path.isDirectory
import kotlin.io.path.listDirectoryEntries
import kotlin.io.path.name

/** A folder of the destination with files of one place from other days, relative to the root. */
internal class NearbyFolder(
    val key: PlaceKey,
    val dir: Path,
    /** Each day of the folder and the folder that holds its files. */
    val days: List<Pair<LocalDate, Path>>,
)

/**
 * Finds folders of each place in its months and the months next to them, and counts their days in
 * the place's span. Dubai on March 4 and March 7 then share `2024/3/Dubai`.
 */
internal fun PlaceFolders.gatherNearby(root: Path): List<NearbyFolder> {
    val probe = DirProbe(root)
    val found = mutableListOf<NearbyFolder>()
    for ((key, span) in places) {
        for (month in nearbyMonths(span.days.months())) {
            for (layout in layouts(key.country, key.place)) {
                for ((dir, days) in foldersInMonth(probe, key.year, month, layout)) found += NearbyFolder(key, dir, days)
            }
        }
    }
    for (folder in found) {
        for ((date, _) in folder.days) {
            places[folder.key]?.days?.add(date)
            folder.key.country?.let { countries[CountryKey(folder.key.year, it)]?.days?.add(date) }
        }
    }
    for (span in countries.values + places.values) span.level = span.days.level()
    return found
}

private fun nearbyMonths(months: Set<Int>): Set<Int> =
    months.flatMapTo(sortedSetOf()) { (it - 1)..(it + 1) }.filterTo(sortedSetOf()) { it in 1..12 }

/** Each way an earlier run could have laid out the place: in a month or a day folder, its country at the same level or above. */
private fun layouts(country: String?, place: String): List<List<Pair<String, PlaceLevel>>> = buildList {
    for (placeLevel in listOf(PlaceLevel.MONTH, PlaceLevel.DAY)) {
        if (country == null) {
            add(listOf(place to placeLevel))
            continue
        }
        for (countryLevel in PlaceLevel.entries.filter { it <= placeLevel }) {
            add(listOf(country to countryLevel, place to placeLevel))
        }
    }
}

private fun foldersInMonth(
    probe: DirProbe,
    year: Int,
    month: Int,
    layout: List<Pair<String, PlaceLevel>>,
): List<Pair<Path, List<Pair<LocalDate, Path>>>> {
    val first = LocalDate.of(year, month, 1)
    return when (layout.last().second) {
        PlaceLevel.DAY -> (1..first.lengthOfMonth()).mapNotNull { day ->
            val date = first.withDayOfMonth(day)
            val dir = datedDir(date, layout, true)
            if (probe.isDir(dir)) dir to listOf(date to dir) else null
        }
        PlaceLevel.MONTH -> {
            val dir = datedDir(first, layout, true)
            val days = if (probe.isDir(dir)) dayFolders(probe.root, dir, first) else emptyList()
            if (days.isEmpty()) emptyList() else listOf(dir to days)
        }
        PlaceLevel.YEAR -> emptyList()
    }
}

/** The `<day>` folders inside a month's place folder. */
private fun dayFolders(root: Path, dir: Path, first: LocalDate): List<Pair<LocalDate, Path>> {
    val entries = try {
        root.resolve(dir).listDirectoryEntries()
    } catch (_: IOException) {
        return emptyList()
    }
    return entries
        .filter { it.isDirectory() }
        .mapNotNull { entry ->
            val day = entry.name.toIntOrNull()?.takeIf { it in 1..first.lengthOfMonth() } ?: return@mapNotNull null
            first.withDayOfMonth(day) to dir.resolve(entry.name)
        }
        .sortedWith(compareBy<Pair<LocalDate, Path>> { it.first }.thenBy { it.second })
}
