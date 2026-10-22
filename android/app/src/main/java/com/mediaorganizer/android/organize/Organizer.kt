package com.mediaorganizer.android.organize

import com.mediaorganizer.android.places.PlaceSources
import java.nio.file.Path

data class OrganizeRequest(val sourceRoot: Path, val destinationRoot: Path)

data class MovedFile(val from: Path, val to: Path)

/**
 * Scans [request], reads every date, then sorts each photo and video, reporting into [progress].
 * With [places], files go to place folders as on desktop and earlier days of those places join them.
 */
fun organize(
    request: OrganizeRequest,
    readMetadata: MetadataReader,
    progress: RunProgress,
    places: PlaceSources? = null,
    isCancelled: () -> Boolean,
): List<MovedFile> {
    progress.log("Scanning ${request.sourceRoot}")
    val scan = scanMedia(request.sourceRoot, request.destinationRoot)
    progress.scanned(scan.media.size, scan.skipped)
    scan.errors.forEach(progress::log)
    progress.log("Found ${scan.media.size} photos and videos, moving to ${request.destinationRoot}")

    val root = request.destinationRoot
    val dated = readDates(scan.media, readMetadata, progress, isCancelled)
    val layout = dated?.let { placeLayoutOf(it, request, places, progress) }
    val library = layout?.let { moveLibrary(it.libraryMoves, root, progress, isCancelled) }.orEmpty()

    val moved = mutableListOf<MovedFile>()
    var end = if (dated == null) RunEnd.STOPPED else RunEnd.DONE
    for (file in dated.orEmpty()) {
        if (isCancelled()) {
            end = RunEnd.STOPPED
            break
        }
        val result = try {
            moveDated(file, root, folderOf(file, root, layout))
        } catch (error: Exception) {
            FileResult(FileOutcome.FAILED, "${file.file.path.fileName}: $error")
        }
        result.movedTo?.let { moved += MovedFile(file.file.path, it) }
        progress.record(result.outcome, result.message)
    }

    removeEmptyFolders(request.sourceRoot, moved.mapNotNull { it.from.parent }.toSet())
    progress.log(if (end == RunEnd.DONE) "Done" else "Stopped")
    progress.finish(end)
    return library + moved
}

/** Reads the date of every file and records the files without one. Null when the run is cancelled. */
private fun readDates(
    media: List<MediaFile>,
    readMetadata: MetadataReader,
    progress: RunProgress,
    isCancelled: () -> Boolean,
): List<DatedFile>? {
    val dated = mutableListOf<DatedFile>()
    for (file in media) {
        if (isCancelled()) return null
        val read = try {
            readDate(file, readMetadata)
        } catch (error: Exception) {
            FileResult(FileOutcome.FAILED, "${file.path.fileName}: $error")
        }
        when (read) {
            is DatedFile -> dated += read
            is FileResult -> progress.record(read.outcome, read.message)
        }
    }
    return dated
}
