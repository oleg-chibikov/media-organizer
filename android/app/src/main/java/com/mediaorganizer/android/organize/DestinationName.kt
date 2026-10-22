package com.mediaorganizer.android.organize

import java.nio.file.Path
import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.util.Locale

private val stemDateFormat = DateTimeFormatter.ofPattern("uuuu-MM-dd HH-mm-ss", Locale.ROOT)

/** `<root>/<year>/<month>/<day>`, month and day without leading zeros as on desktop. */
fun datedDirectory(root: Path, date: LocalDateTime): Path = root
    .resolve(String.format(Locale.ROOT, "%04d", date.year))
    .resolve(date.monthValue.toString())
    .resolve(date.dayOfMonth.toString())

/**
 * `<date>[ <place>, <country>] (<original>)[ - <device>]`. [originalName] must already be sanitized.
 * The place sits before the original name, so a place with brackets doesn't hide it from [parseOriginalName].
 */
fun destinationStem(date: LocalDateTime, placeLabel: String?, originalName: String, deviceName: String?): String {
    val dated = date.format(stemDateFormat)
    val stem = if (placeLabel == null) "$dated ($originalName)" else "$dated $placeLabel ($originalName)"
    return if (deviceName == null) stem else "$stem - ${sanitizeForWindowsFilename(deviceName)}"
}

/** `<place>, <country>` for a file name, or one of them when the other is empty or the same. */
fun placeLabel(place: String, country: String): String? {
    val placeName = place.trim()
    val countryName = country.trim()
    val label = when {
        placeName.isEmpty() && countryName.isEmpty() -> return null
        countryName.isEmpty() || placeName == countryName -> placeName
        placeName.isEmpty() -> countryName
        else -> "$placeName, $countryName"
    }
    return sanitizeForWindowsFilename(label)
}
