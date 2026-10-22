package com.mediaorganizer.android.places

import java.nio.file.Path
import java.time.Duration
import java.time.LocalDateTime

/**
 * A file with no GPS takes the place of the closest geotagged file from the same device within
 * 3 hours. A file with no device name, or none close, stays out.
 */
internal fun untaggedPlaces(shots: List<Shot>, tagged: Map<Path, FilePlace>): Map<Path, FilePlace> {
    // Shots are in time order, so each device's list is too.
    val byDevice = HashMap<String, MutableList<Pair<LocalDateTime, FilePlace>>>()
    for (shot in shots) {
        val device = shot.device ?: continue
        val place = tagged[shot.path] ?: continue
        byDevice.getOrPut(device) { mutableListOf() } += shot.taken to place
    }
    return shots
        .filter { it.location == null }
        .mapNotNull { shot ->
            val anchors = byDevice[shot.device ?: return@mapNotNull null] ?: return@mapNotNull null
            val (taken, place) = nearest(anchors, shot.taken) ?: return@mapNotNull null
            if (gap(taken, shot.taken) <= UNTAGGED_MAX_GAP) shot.path to place else null
        }
        .toMap()
}

/** The anchor closest to [taken] in a list sorted by time. */
private fun nearest(anchors: List<Pair<LocalDateTime, FilePlace>>, taken: LocalDateTime): Pair<LocalDateTime, FilePlace>? {
    // The first anchor taken after [taken]. A binary search, since anchors are in time order.
    val after = anchors.binarySearch { if (it.first <= taken) -1 else 1 }.let { -(it + 1) }
    val before = anchors.getOrNull(after - 1)
    return listOfNotNull(before, anchors.getOrNull(after)).minByOrNull { gap(it.first, taken) }
}

private fun gap(a: LocalDateTime, b: LocalDateTime): Duration = Duration.between(a, b).abs()
