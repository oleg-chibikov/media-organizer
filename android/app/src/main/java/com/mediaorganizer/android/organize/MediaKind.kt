package com.mediaorganizer.android.organize

enum class MediaKind { PHOTO, VIDEO }

private val photoExtensions = setOf("jpg", "jpeg", "png", "heic", "heif", "tif", "tiff", "webp")
private val videoExtensions = setOf("mp4", "mov", "m4v", "avi", "mts", "m2ts")

/** Same extension lists as the desktop scanner. Other files stay where they are. */
fun mediaKindOf(extension: String): MediaKind? = when (extension.lowercase()) {
    in photoExtensions -> MediaKind.PHOTO
    in videoExtensions -> MediaKind.VIDEO
    else -> null
}
