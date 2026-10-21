package com.mediaorganizer.android.metadata

import java.time.LocalDateTime
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeParseException
import java.util.Locale

private val retrieverFormat = DateTimeFormatter.ofPattern("uuuuMMdd'T'HHmmss", Locale.ROOT)
private val exifToolFormat = DateTimeFormatter.ofPattern("uuuu:MM:dd HH:mm:ss", Locale.ROOT)
private const val EMPTY_MOVIE_DATE_YEAR = 1904

/**
 * Turns `20240501T101500.000Z` into `2024:05:01 10:15:00`. ExifTool prints the movie header
 * date as UTC wall time without a zone, and the desktop app reads it as local time.
 * Matching that keeps both apps putting a video in the same folder.
 */
internal fun retrieverDateToExifTool(value: String?): String? {
    if (value == null || value.length < 15) return null
    val date = try {
        LocalDateTime.parse(value.substring(0, 15), retrieverFormat)
    } catch (_: DateTimeParseException) {
        return null
    }
    // A zero movie header date comes back as 1904-01-01; ExifTool prints it as 0000:00:00.
    return if (date.year == EMPTY_MOVIE_DATE_YEAR) null else date.format(exifToolFormat)
}
