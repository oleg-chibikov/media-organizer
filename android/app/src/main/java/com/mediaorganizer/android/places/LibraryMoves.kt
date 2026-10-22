package com.mediaorganizer.android.places

import java.nio.file.Files
import java.nio.file.Path

/** A photo or video of an earlier run that moves into its place's new folder, keeping its name. */
data class LibraryMove(val from: Path, val toDirectory: Path)

/**
 * Each photo and video of the [nearby] folders, with the folder of its day under the new levels:
 * `2024/3/4/Dubai/a.jpg` goes to `2024/3/Dubai/4/a.jpg`. Subfolders come along.
 */
internal fun PlaceFolders.libraryMoves(root: Path, nearby: List<NearbyFolder>, isMedia: (Path) -> Boolean): List<LibraryMove> =
    nearby.flatMap { folder ->
        val target = keyFolders(folder.key)
        folder.days.flatMap { (date, dayDir) ->
            val dayTarget = datedDir(date, target, false)
            if (dayTarget == dayDir) return@flatMap emptyList()
            val from = root.resolve(dayDir)
            val to = root.resolve(dayTarget)
            filesUnder(from).filter(isMedia).map { file ->
                LibraryMove(file, from.relativize(file).parent?.let(to::resolve) ?: to)
            }
        }
    }

private fun filesUnder(folder: Path): List<Path> =
    Files.walk(folder).use { paths -> paths.iterator().asSequence().filter { Files.isRegularFile(it) }.toList() }
