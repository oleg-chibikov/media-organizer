package com.mediaorganizer.android.organize

import java.time.LocalDateTime
import java.time.ZoneId

sealed interface CaptureDate {
    data class Found(val date: LocalDateTime, val sourceTag: String) : CaptureDate
    data object Missing : CaptureDate
    data class Unparseable(val message: String) : CaptureDate
}

private const val FALLBACK_TAG = "FileModifyDate"

/** Picks the first parseable date in the file kind's tag priority order. */
fun selectCaptureDate(kind: MediaKind, raw: RawMetadata, zone: ZoneId = ZoneId.systemDefault()): CaptureDate {
    val failures = mutableListOf<String>()
    for ((tag, value) in priority(kind, raw)) {
        if (value == null) continue
        val date = parseMetadataTimestamp(value, zone)
        if (date != null) return CaptureDate.Found(date, tag)
        failures += "$tag=$value"
    }
    if (failures.isEmpty()) return CaptureDate.Missing
    return CaptureDate.Unparseable("Could not parse metadata date values in priority order: ${failures.joinToString()}")
}

private fun priority(kind: MediaKind, raw: RawMetadata): List<Pair<String, String?>> = when (kind) {
    MediaKind.PHOTO -> listOf(
        "SubSecDateTimeOriginal" to raw.subSecDateTimeOriginal,
        "DateTimeOriginal" to raw.dateTimeOriginal,
        "CreateDate" to raw.createDate,
        "ModifyDate" to raw.modifyDate,
        FALLBACK_TAG to raw.fileModifyDate,
    )
    MediaKind.VIDEO -> listOf(
        "CreateDate" to raw.createDate,
        "ModifyDate" to raw.modifyDate,
        FALLBACK_TAG to raw.fileModifyDate,
    )
}
