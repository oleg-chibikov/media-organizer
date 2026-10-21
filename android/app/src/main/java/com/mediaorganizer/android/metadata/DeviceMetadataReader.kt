package com.mediaorganizer.android.metadata

import com.mediaorganizer.android.organize.MediaKind
import com.mediaorganizer.android.organize.MetadataReader
import com.mediaorganizer.android.organize.RawMetadata
import java.nio.file.Files
import java.nio.file.Path
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.Locale

private val fileDateFormat = DateTimeFormatter.ofPattern("uuuu:MM:dd HH:mm:ssxxx", Locale.ROOT)

/** Reads the tags the desktop app asks ExifTool for, plus the file date as the last fallback. */
object DeviceMetadataReader : MetadataReader {
    override fun read(path: Path, kind: MediaKind): RawMetadata {
        val tags = when (kind) {
            MediaKind.PHOTO -> readPhotoMetadata(path)
            MediaKind.VIDEO -> readVideoMetadata(path)
        }
        val modified = Files.getLastModifiedTime(path).toInstant().atZone(ZoneId.systemDefault())
        return tags.copy(fileModifyDate = modified.format(fileDateFormat))
    }
}
