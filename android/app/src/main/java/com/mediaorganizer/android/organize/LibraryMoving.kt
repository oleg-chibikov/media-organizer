package com.mediaorganizer.android.organize

import com.mediaorganizer.android.places.LibraryMove
import java.nio.file.Files
import java.nio.file.Path
import kotlin.io.path.extension
import kotlin.io.path.name
import kotlin.io.path.nameWithoutExtension

/**
 * Moves files of earlier runs into their place's new folder, keeping their names, then removes the
 * folders that emptied. A file with the same bytes already there stays.
 */
internal fun moveLibrary(
    moves: List<LibraryMove>,
    root: Path,
    progress: RunProgress,
    isCancelled: () -> Boolean,
): List<MovedFile> {
    if (moves.isEmpty()) return emptyList()
    progress.log("Gathering ${moves.size} files of earlier days into their place folders")
    val moved = mutableListOf<MovedFile>()
    for (move in moves) {
        if (isCancelled()) break
        try {
            moveKeepingName(move, root, progress)?.let { moved += it }
        } catch (error: Exception) {
            progress.log("${move.from.name}: $error")
        }
    }
    removeEmptyFolders(root, moved.mapNotNull { it.from.parent }.toSet())
    return moved
}

private fun moveKeepingName(move: LibraryMove, root: Path, progress: RunProgress): MovedFile? {
    val from = move.from
    val destination = allocateDestination(from, move.toDirectory, from.nameWithoutExtension, from.extension)
    val shown = root.relativize(destination.path)
    if (destination is Destination.Duplicate) {
        progress.log("Already there: ${root.relativize(from)} = $shown")
        return null
    }
    Files.createDirectories(move.toDirectory)
    moveWithoutReplacing(from, destination.path)
    progress.log("${root.relativize(from)} -> $shown")
    return MovedFile(from, destination.path)
}
