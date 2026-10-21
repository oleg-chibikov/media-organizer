package com.mediaorganizer.android.organize

import java.nio.file.Files
import java.nio.file.Path
import kotlin.io.path.extension
import kotlin.io.path.name

fun interface MetadataReader {
    fun read(path: Path, kind: MediaKind): RawMetadata
}

data class FileResult(val outcome: FileOutcome, val message: String, val movedTo: Path? = null)

/** Plans one file the way the desktop planner does, then moves it without replacing anything. */
internal fun organizeFile(file: MediaFile, destinationRoot: Path, readMetadata: MetadataReader): FileResult {
    val name = file.path.name
    val metadata = readMetadata.read(file.path, file.kind)
    val date = when (val chosen = selectCaptureDate(file.kind, metadata)) {
        is CaptureDate.Found -> chosen.date
        CaptureDate.Missing -> return FileResult(FileOutcome.NO_DATE, "No date: $name")
        is CaptureDate.Unparseable -> return FileResult(FileOutcome.FAILED, "$name: ${chosen.message}")
    }

    val originalName = sanitizeForWindowsFilename(parseOriginalName(name))
    val stem = destinationStem(date, originalName, metadata.deviceName)
    val directory = datedDirectory(destinationRoot, date)
    val destination = allocateDestination(file.path, directory, stem, file.path.extension)
    val shownDestination = destinationRoot.relativize(destination.path)

    if (destination is Destination.Duplicate) {
        return FileResult(FileOutcome.DUPLICATE, "Already there: $name = $shownDestination")
    }
    Files.createDirectories(directory)
    // Without REPLACE_EXISTING the move fails if another file took the name in the meantime.
    Files.move(file.path, destination.path)
    return FileResult(FileOutcome.MOVED, "$name -> $shownDestination", destination.path)
}
