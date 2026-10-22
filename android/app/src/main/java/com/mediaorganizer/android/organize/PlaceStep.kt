package com.mediaorganizer.android.organize

import com.mediaorganizer.android.places.PlaceInput
import com.mediaorganizer.android.places.PlaceLayout
import com.mediaorganizer.android.places.PlaceSources
import com.mediaorganizer.android.places.planPlaces
import java.nio.file.Path
import kotlin.io.path.extension
import kotlin.io.path.name

/** The place folders of a run's files. Null when place folders are off, or failed and the files keep date folders. */
internal fun placeLayoutOf(dated: List<DatedFile>, root: Path, places: PlaceSources?, progress: RunProgress): PlaceLayout? {
    if (places == null || dated.isEmpty()) return null
    progress.log("Finding places")
    val files = dated.map { PlaceInput(it.file.path, it.date, it.metadata.deviceName, it.metadata.gps) }
    return try {
        planPlaces(files, root, places, ::isVisibleMedia)
    } catch (error: Exception) {
        progress.log("Place folders failed, using date folders: $error")
        null
    }
}

/** The place folder of [file], else `<year>/<month>/<day>`. */
internal fun folderOf(file: DatedFile, root: Path, layout: PlaceLayout?): Path =
    layout?.folders?.relativeDir(file.file.path, file.date.toLocalDate())?.let(root::resolve)
        ?: datedDirectory(root, file.date)

private fun isVisibleMedia(path: Path) = !path.name.startsWith(".") && mediaKindOf(path.extension) != null
