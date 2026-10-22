package com.mediaorganizer.android.places

import java.nio.file.Files
import java.nio.file.Path

/**
 * Raises each country and place folder to a higher date level where [root] already has it. One day
 * in Osaka then joins `2024/3/Osaka` instead of a new `2024/3/15/Osaka`.
 */
internal fun PlaceFolders.reuseExisting(root: Path) {
    for ((key, span) in countries) {
        val date = span.days.firstDate(key.year) ?: continue
        span.level = existingLevel(root, span.level) { level -> datedDir(date, listOf(key.country to level), true) }
            ?: span.level
    }
    for ((key, span) in places) {
        val date = span.days.firstDate(key.year) ?: continue
        val country = key.country?.let { it to countries.getValue(CountryKey(key.year, it)).level }
        span.level = existingLevel(root, span.level) { level ->
            // A place folder sits at or below its country folder.
            if (country != null && level < country.second) null
            else datedDir(date, listOfNotNull(country) + (key.place to level), true)
        } ?: span.level
    }
}

/** The highest level above [current] whose folder is already under [root]. */
private fun existingLevel(root: Path, current: PlaceLevel, folderAt: (PlaceLevel) -> Path?): PlaceLevel? =
    PlaceLevel.entries
        .filter { it < current }
        .firstOrNull { level -> folderAt(level)?.let { Files.isDirectory(root.resolve(it)) } == true }
