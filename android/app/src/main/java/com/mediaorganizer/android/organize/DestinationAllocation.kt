package com.mediaorganizer.android.organize

import java.nio.file.Path
import kotlin.io.path.exists

sealed interface Destination {
    val path: Path

    data class Free(override val path: Path) : Destination

    /** A file with the same bytes is already there, so the source stays put. */
    data class Duplicate(override val path: Path) : Destination
}

/** Picks the first free `<stem>[ (n)].<ext>` in [directory]. */
fun allocateDestination(source: Path, directory: Path, stem: String, extension: String): Destination {
    var suffix = 0
    while (true) {
        val name = buildString {
            append(stem)
            if (suffix > 0) append(" ($suffix)")
            if (extension.isNotEmpty()) append(".").append(extension)
        }
        val candidate = directory.resolve(name)
        if (!candidate.exists()) return Destination.Free(candidate)
        if (filesHaveSameBytes(source, candidate)) return Destination.Duplicate(candidate)
        suffix++
    }
}
