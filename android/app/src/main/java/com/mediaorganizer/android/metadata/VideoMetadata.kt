package com.mediaorganizer.android.metadata

import android.media.MediaMetadataRetriever
import com.mediaorganizer.android.organize.RawMetadata
import java.nio.file.Path

internal fun readVideoMetadata(path: Path): RawMetadata {
    val retriever = MediaMetadataRetriever()
    return try {
        retriever.setDataSource(path.toString())
        RawMetadata(
            createDate = retrieverDateToExifTool(retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_DATE)),
            gps = parseIso6709(retriever.extractMetadata(MediaMetadataRetriever.METADATA_KEY_LOCATION)),
        )
    } catch (_: RuntimeException) {
        RawMetadata()
    } finally {
        retriever.release()
    }
}
