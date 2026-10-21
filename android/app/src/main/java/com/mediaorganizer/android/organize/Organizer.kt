package com.mediaorganizer.android.organize

import java.nio.file.Path

data class OrganizeRequest(val sourceRoot: Path, val destinationRoot: Path)

data class MovedFile(val from: Path, val to: Path)

/** Scans [request] then sorts each photo and video, reporting into [progress]. */
fun organize(
    request: OrganizeRequest,
    readMetadata: MetadataReader,
    progress: RunProgress,
    isCancelled: () -> Boolean,
): List<MovedFile> {
    progress.log("Scanning ${request.sourceRoot}")
    val scan = scanMedia(request.sourceRoot, request.destinationRoot)
    progress.scanned(scan.media.size, scan.skipped)
    scan.errors.forEach(progress::log)
    progress.log("Found ${scan.media.size} photos and videos, moving to ${request.destinationRoot}")

    val moved = mutableListOf<MovedFile>()
    var end = RunEnd.DONE
    for (file in scan.media) {
        if (isCancelled()) {
            end = RunEnd.STOPPED
            break
        }
        val result = try {
            organizeFile(file, request.destinationRoot, readMetadata)
        } catch (error: Exception) {
            FileResult(FileOutcome.FAILED, "${file.path.fileName}: $error")
        }
        result.movedTo?.let { moved += MovedFile(file.path, it) }
        progress.record(result.outcome, result.message)
    }

    removeEmptySourceFolders(request.sourceRoot, moved.mapNotNull { it.from.parent }.toSet())
    progress.log(if (end == RunEnd.DONE) "Done" else "Stopped")
    progress.finish(end)
    return moved
}
