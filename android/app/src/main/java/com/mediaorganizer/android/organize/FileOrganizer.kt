package com.mediaorganizer.android.organize

import java.nio.file.Files
import java.nio.file.Path
import java.time.LocalDateTime
import kotlin.io.path.extension
import kotlin.io.path.name

fun interface MetadataReader {
    fun read(path: Path, kind: MediaKind): RawMetadata
}

/** What reading a file's date gives: the file with its date, or the result that ends it. */
sealed interface DateRead

data class FileResult(val outcome: FileOutcome, val message: String, val movedTo: Path? = null) : DateRead

/** A photo or video with its capture date, ready for a folder. */
class DatedFile(val file: MediaFile, val date: LocalDateTime, val metadata: RawMetadata) : DateRead

/** Picks the capture date the way the desktop planner does. */
internal fun readDate(file: MediaFile, readMetadata: MetadataReader): DateRead {
    val metadata = readMetadata.read(file.path, file.kind)
    return when (val chosen = selectCaptureDate(file.kind, metadata)) {
        is CaptureDate.Found -> DatedFile(file, chosen.date, metadata)
        CaptureDate.Missing -> FileResult(FileOutcome.NO_DATE, "No date: ${file.path.name}")
        is CaptureDate.Unparseable -> FileResult(FileOutcome.FAILED, "${file.path.name}: ${chosen.message}")
    }
}

/** Moves [dated] into [directory] under its desktop file name, without replacing anything. */
internal fun moveDated(dated: DatedFile, destinationRoot: Path, directory: Path): FileResult {
    val path = dated.file.path
    val originalName = sanitizeForWindowsFilename(parseOriginalName(path.name))
    val stem = destinationStem(dated.date, originalName, dated.metadata.deviceName)
    val destination = allocateDestination(path, directory, stem, path.extension)
    val shownDestination = destinationRoot.relativize(destination.path)

    if (destination is Destination.Duplicate) {
        return FileResult(FileOutcome.DUPLICATE, "Already there: ${path.name} = $shownDestination")
    }
    Files.createDirectories(directory)
    moveWithoutReplacing(path, destination.path)
    return FileResult(FileOutcome.MOVED, "${path.name} -> $shownDestination", destination.path)
}
