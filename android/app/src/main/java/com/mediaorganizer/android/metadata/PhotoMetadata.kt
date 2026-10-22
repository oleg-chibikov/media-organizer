package com.mediaorganizer.android.metadata

import android.media.ExifInterface
import com.mediaorganizer.android.geo.Coordinates
import com.mediaorganizer.android.organize.RawMetadata
import java.io.IOException
import java.nio.file.Path

/** EXIF tags under ExifTool's names. `CreateDate` is EXIF `DateTimeDigitized`. */
internal fun readPhotoMetadata(path: Path): RawMetadata {
    val exif = try {
        ExifInterface(path.toString())
    } catch (_: IOException) {
        return RawMetadata()
    }
    val original = exif.tag(ExifInterface.TAG_DATETIME_ORIGINAL)
    return RawMetadata(
        subSecDateTimeOriginal = original?.let {
            subSecDate(it, exif.tag(ExifInterface.TAG_SUBSEC_TIME_ORIGINAL), exif.tag(ExifInterface.TAG_OFFSET_TIME_ORIGINAL))
        },
        dateTimeOriginal = original,
        createDate = exif.tag(ExifInterface.TAG_DATETIME_DIGITIZED),
        modifyDate = exif.tag(ExifInterface.TAG_DATETIME),
        deviceName = exif.tag(ExifInterface.TAG_MODEL) ?: exif.tag(ExifInterface.TAG_MAKE),
        gps = exif.gps(),
    )
}

/** The EXIF GPS point. Android gives it as floats, about a metre apart. */
private fun ExifInterface.gps(): Coordinates? {
    val point = FloatArray(2)
    return if (getLatLong(point)) gpsPoint(point[0].toDouble(), point[1].toDouble()) else null
}

/** Builds ExifTool's `SubSecDateTimeOriginal`: the date, then `.subsec`, then the offset. */
internal fun subSecDate(dateTime: String, subSec: String?, offset: String?): String =
    dateTime + subSec?.let { ".$it" }.orEmpty() + offset.orEmpty()

private fun ExifInterface.tag(name: String): String? = getAttribute(name)?.trim()?.ifEmpty { null }
