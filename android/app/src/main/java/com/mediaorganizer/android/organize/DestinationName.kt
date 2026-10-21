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

/** `<date> (<original>)[ - <device>]`. [originalName] must already be sanitized. */
fun destinationStem(date: LocalDateTime, originalName: String, deviceName: String?): String {
    val stem = "${date.format(stemDateFormat)} ($originalName)"
    return if (deviceName == null) stem else "$stem - ${sanitizeForWindowsFilename(deviceName)}"
}
