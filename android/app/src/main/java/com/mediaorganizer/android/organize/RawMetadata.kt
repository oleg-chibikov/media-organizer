package com.mediaorganizer.android.organize

import com.mediaorganizer.android.geo.Coordinates

/**
 * The tags the desktop app reads with ExifTool, as ExifTool prints them:
 * `yyyy:MM:dd HH:mm:ss[.f][±HH:MM]`. Video `MediaCreateDate` and `TrackCreateDate` are left out
 * because Android exposes only the movie header date, which ExifTool calls `CreateDate`.
 */
data class RawMetadata(
    val subSecDateTimeOriginal: String? = null,
    val dateTimeOriginal: String? = null,
    val createDate: String? = null,
    val modifyDate: String? = null,
    val fileModifyDate: String? = null,
    val deviceName: String? = null,
    val gps: Coordinates? = null,
)
