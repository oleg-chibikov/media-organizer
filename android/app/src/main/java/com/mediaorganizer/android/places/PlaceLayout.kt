package com.mediaorganizer.android.places

import java.nio.file.Path

/** The place folders of one run, the files of earlier runs that move to join them, and what detection found. */
class PlaceLayout(val folders: PlaceFolders, val libraryMoves: List<LibraryMove>, val detection: Detection)

/**
 * Finds each file's place, then fits the folders to [root] as the desktop app does: earlier days of
 * a place nearby count toward its level, and folders the root already has win when they sit higher.
 */
fun planPlaces(files: List<PlaceInput>, root: Path, sources: PlaceSources, isMedia: (Path) -> Boolean): PlaceLayout {
    val detection = detectFilePlaces(PlaceScan(files, sources.geocode), sources.homes)
    val folders = PlaceFolders.of(detection.filePlaces, files.associate { it.path to it.taken.toLocalDate() })
    val nearby = folders.gatherNearby(root)
    folders.reuseExisting(root)
    return PlaceLayout(folders, folders.libraryMoves(root, nearby, isMedia), detection)
}
