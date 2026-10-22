package com.mediaorganizer.android.places

import java.nio.file.Files
import java.nio.file.Path

/**
 * Checks folders under [root] once each, and skips a folder whose parent is missing. A search over
 * every day of a month then checks each missing day folder once.
 */
internal class DirProbe(val root: Path) {
    private val known = HashMap<Path, Boolean>()

    fun isDir(relative: Path): Boolean {
        known[relative]?.let { return it }
        val parentFound = relative.parent?.let(::isDir) ?: true
        val found = parentFound && Files.isDirectory(root.resolve(relative))
        known[relative] = found
        return found
    }
}
