package com.mediaorganizer.android.organize

import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.attribute.BasicFileAttributes
import java.nio.file.attribute.FileTime
import java.time.Duration
import java.time.Instant

/** A camera writes a video for minutes, and some cameras rewrite a photo seconds after saving it. */
val SETTLE_TIME: Duration = Duration.ofSeconds(30)

/** Size and change time, to tell a finished file from one an app still writes. */
data class FileState(val size: Long, val modified: FileTime) {
    /** A change time in the future comes from a wrong clock, not from a writer, so that file is settled. */
    fun isFresh(now: Instant): Boolean {
        val age = Duration.between(modified.toInstant(), now)
        return !age.isNegative && age < SETTLE_TIME
    }

    companion object {
        fun of(path: Path): FileState {
            val attributes = Files.readAttributes(path, BasicFileAttributes::class.java)
            return FileState(attributes.size(), attributes.lastModifiedTime())
        }
    }
}
