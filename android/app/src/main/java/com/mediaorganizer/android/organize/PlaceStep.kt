package com.mediaorganizer.android.organize

import com.mediaorganizer.android.places.Detection
import com.mediaorganizer.android.places.HomeSource
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
        planPlaces(files, root, places, ::isVisibleMedia).also { logDetection(it.detection, files.size, progress) }
    } catch (error: Exception) {
        progress.log("Place folders failed, using date folders: $error")
        null
    }
}

private fun logDetection(detection: Detection, total: Int, progress: RunProgress) {
    val home = detection.home
    progress.log(if (home == null) "Home: not set" else "Home: ${home.name}, ${home.country}, ${sourceText(home.source)}")
    val placed = detection.filePlaces.size
    progress.log(if (placed == 0) "No GPS in these files" else "$placed of $total files have a place")
}

private fun sourceText(source: HomeSource) = when (source) {
    HomeSource.MANUAL -> "picked by you"
    HomeSource.DEVICE -> "from your location"
    HomeSource.LEARNED -> "learned from photos"
    HomeSource.TIME_ZONE -> "from time zone"
}

/** The place folder of [file], else `<year>/<month>/<day>`. */
internal fun folderOf(file: DatedFile, root: Path, layout: PlaceLayout?): Path =
    layout?.folders?.relativeDir(file.file.path, file.date.toLocalDate())?.let(root::resolve)
        ?: datedDirectory(root, file.date)

private fun isVisibleMedia(path: Path) = !path.name.startsWith(".") && mediaKindOf(path.extension) != null
