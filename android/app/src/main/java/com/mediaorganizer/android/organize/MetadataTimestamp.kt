package com.mediaorganizer.android.organize

import java.time.LocalDateTime
import java.time.OffsetDateTime
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.format.DateTimeFormatterBuilder
import java.time.format.DateTimeParseException
import java.time.format.ResolverStyle
import java.time.temporal.ChronoField

private fun exifDateFormat(offsetPattern: String?): DateTimeFormatter {
    val builder = DateTimeFormatterBuilder()
        .appendPattern("uuuu:MM:dd HH:mm:ss")
        .optionalStart()
        .appendFraction(ChronoField.NANO_OF_SECOND, 1, 9, true)
        .optionalEnd()
    if (offsetPattern != null) builder.appendOffset(offsetPattern, "Z")
    return builder.toFormatter().withResolverStyle(ResolverStyle.STRICT)
}

private val zonedFormats = listOf(exifDateFormat("+HH:MM"), exifDateFormat("+HHMM"))
private val naiveFormat = exifDateFormat(null)

/**
 * Parses an ExifTool timestamp into local wall time. A zoned value converts to [zone];
 * a value without a zone is taken as local time and fails inside a daylight-saving gap.
 */
fun parseMetadataTimestamp(raw: String, zone: ZoneId): LocalDateTime? {
    zonedFormats.firstNotNullOfOrNull { format -> parseOrNull { OffsetDateTime.parse(raw, format) } }
        ?.let { return it.atZoneSameInstant(zone).toLocalDateTime() }

    val naive = parseOrNull { LocalDateTime.parse(raw, naiveFormat) } ?: return null
    return if (zone.rules.getValidOffsets(naive).isEmpty()) null else naive
}

private inline fun <T> parseOrNull(parse: () -> T): T? =
    try {
        parse()
    } catch (_: DateTimeParseException) {
        null
    }
