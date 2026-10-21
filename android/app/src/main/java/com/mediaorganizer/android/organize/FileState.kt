package com.mediaorganizer.android.organize

import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.attribute.BasicFileAttributes
import java.nio.file.attribute.FileTime

/** Size and change time, to tell whether an app changed a file during a copy. */
data class FileState(val size: Long, val modified: FileTime) {
    companion object {
        fun of(path: Path): FileState {
            val attributes = Files.readAttributes(path, BasicFileAttributes::class.java)
            return FileState(attributes.size(), attributes.lastModifiedTime())
        }
    }
}
