package com.mediaorganizer.android.places

import com.mediaorganizer.android.organize.sanitizeForWindowsFilename
import java.nio.file.Path
import java.time.LocalDate

/** The days a country or place folder's files span within one year, and its date level. */
internal class FolderSpan(val days: DaySpan, var level: PlaceLevel = days.level())

internal data class CountryKey(val year: Int, val country: String)

internal data class PlaceKey(val year: Int, val country: String?, val place: String)

/**
 * The place folder of each file, under `<year>` and the date folders the country and the place span
 * within that year. Files in home's city have no place folders.
 */
class PlaceFolders private constructor(
    private val byFile: Map<Path, PlaceKey>,
    internal val countries: Map<CountryKey, FolderSpan>,
    internal val places: Map<PlaceKey, FolderSpan>,
) {
    /** `<year>/<month>/<day>` with the country and place folders at their level, or null for no place folder. */
    fun relativeDir(path: Path, date: LocalDate): Path? = byFile[path]?.let { datedDir(date, keyFolders(it), false) }

    /** The country folder, when there is one, then the place folder, each with its level. */
    internal fun keyFolders(key: PlaceKey): List<Pair<String, PlaceLevel>> = buildList {
        key.country?.let { add(it to countries.getValue(CountryKey(key.year, it)).level) }
        add(key.place to places.getValue(key).level)
    }

    companion object {
        /** Throws [IllegalArgumentException] on a folder name that isn't one plain name. */
        fun of(filePlaces: Map<Path, FilePlace>, dates: Map<Path, LocalDate>): PlaceFolders {
            val byFile = HashMap<Path, PlaceKey>()
            val countries = HashMap<CountryKey, DaySpan>()
            val places = HashMap<PlaceKey, DaySpan>()
            for ((path, found) in filePlaces) {
                val date = dates[path] ?: continue
                if (found.home == HomeMatch.CITY) continue
                val country = found.countryFolder?.let(::validatedFolderName)
                country?.let { countries.getOrPut(CountryKey(date.year, it), ::DaySpan).add(date) }
                val key = PlaceKey(date.year, country, validatedFolderName(found.place))
                places.getOrPut(key, ::DaySpan).add(date)
                byFile[path] = key
            }
            return PlaceFolders(byFile, countries.mapValues { FolderSpan(it.value) }, places.mapValues { FolderSpan(it.value) })
        }
    }
}

private fun validatedFolderName(name: String): String {
    val trimmed = name.trim()
    require(trimmed.isNotEmpty()) { "A place folder name is empty." }
    require('/' !in trimmed && '\\' !in trimmed && !trimmed.all { it == '.' }) {
        "\"$trimmed\" looks like a path. Use a plain folder name."
    }
    require(trimmed.none(Char::isISOControl)) { "\"$trimmed\" has control characters." }
    return sanitizeForWindowsFilename(trimmed)
}
