package com.mediaorganizer.android.places

import com.mediaorganizer.android.geo.Coordinates
import java.nio.file.Path
import java.time.LocalDateTime

/** What place detection needs of one photo or video with a capture date. */
data class PlaceInput(val path: Path, val taken: LocalDateTime, val device: String?, val gps: Coordinates?)

internal class Shot(
    val path: Path,
    val taken: LocalDateTime,
    /** The camera or phone, so a file with no GPS joins a place of the same device. */
    val device: String?,
    /** The place and city of the file's GPS point. */
    val location: Resolved?,
)

/** The files sorted by date, each GPS point geocoded. */
internal fun collectShots(files: List<PlaceInput>, places: PlaceTable): List<Shot> = files
    .map { Shot(it.path, it.taken, it.device?.ifEmpty { null }, it.gps?.let(places::resolve)) }
    .sortedWith(compareBy<Shot> { it.taken }.thenBy { it.path })
