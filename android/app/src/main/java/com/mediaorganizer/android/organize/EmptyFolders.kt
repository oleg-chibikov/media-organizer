package com.mediaorganizer.android.organize

import java.io.IOException
import java.nio.file.Files
import java.nio.file.LinkOption
import java.nio.file.Path
import kotlin.io.path.isDirectory

/** Removes folders a run emptied, walking up to just below [root]. The root itself stays. */
fun removeEmptyFolders(root: Path, folders: Set<Path>) {
    for (folder in folders) {
        var current: Path? = folder
        while (current != null && current != root && current.startsWith(root) && removeIfEmptyFolder(current)) {
            current = current.parent
        }
    }
}

private fun removeIfEmptyFolder(folder: Path): Boolean {
    if (!folder.isDirectory(LinkOption.NOFOLLOW_LINKS)) return false
    return try {
        Files.delete(folder)
        true
    } catch (_: IOException) {
        false
    }
}
