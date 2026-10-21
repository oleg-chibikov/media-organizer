package com.mediaorganizer.android.metadata

import android.os.Build
import com.mediaorganizer.android.organize.MediaKind
import com.mediaorganizer.android.organize.MetadataReader
import com.mediaorganizer.android.organize.RawMetadata
import java.nio.file.Files
import java.nio.file.Path
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.util.Locale

private val fileDateFormat = DateTimeFormatter.ofPattern("uuuu:MM:dd HH:mm:ssxxx", Locale.ROOT)

/**
 * Reads the tags the desktop app asks ExifTool for, plus the file date as the last date fallback.
 * A file without a camera model gets this phone's model.
 */
object DeviceMetadataReader : MetadataReader {
    override fun read(path: Path, kind: MediaKind): RawMetadata {
        val tags = when (kind) {
            MediaKind.PHOTO -> readPhotoMetadata(path)
            MediaKind.VIDEO -> readVideoMetadata(path)
        }
        val modified = Files.getLastModifiedTime(path).toInstant().atZone(ZoneId.systemDefault())
        return tags.copy(
            fileModifyDate = modified.format(fileDateFormat),
            deviceName = tags.deviceName ?: phoneModel,
        )
    }

    // The sorted folders hold what this phone shot. Android exposes no camera model for a video.
    private val phoneModel: String? = Build.MODEL?.trim()?.ifEmpty { null }
}
