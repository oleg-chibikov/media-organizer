package com.mediaorganizer.android.organize

import java.nio.file.AtomicMoveNotSupportedException
import java.nio.file.FileAlreadyExistsException
import java.nio.file.Files
import java.nio.file.LinkOption
import java.nio.file.Path
import java.nio.file.StandardCopyOption

fun interface Rename {
    /** Renames on one volume. Throws [AtomicMoveNotSupportedException] across volumes. */
    fun move(from: Path, to: Path)
}

/**
 * Moves [source] to a free [destination]. On any error the source stays where it was.
 * Across volumes the source goes only after its checked copy holds the destination name.
 */
internal fun moveWithoutReplacing(source: Path, destination: Path, rename: Rename = Rename(::renameIfFree)) {
    try {
        rename.move(source, destination)
    } catch (_: AtomicMoveNotSupportedException) {
        copyThenRemoveSource(source, destination, rename)
    }
}

/** Android has no rename that refuses a taken name, so the check sits right before the rename. */
internal fun renameIfFree(from: Path, to: Path) {
    if (!Files.notExists(to, LinkOption.NOFOLLOW_LINKS)) throw FileAlreadyExistsException(to.toString())
    // ATOMIC_MOVE stops Java from copying and deleting on its own when the volumes differ.
    Files.move(from, to, StandardCopyOption.ATOMIC_MOVE)
}
