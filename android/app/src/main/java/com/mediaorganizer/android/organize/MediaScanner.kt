package com.mediaorganizer.android.organize

import java.io.IOException
import java.nio.file.FileVisitResult
import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.SimpleFileVisitor
import java.nio.file.attribute.BasicFileAttributes
import kotlin.io.path.extension
import kotlin.io.path.name

data class MediaFile(val path: Path, val kind: MediaKind)

data class ScanResult(val media: List<MediaFile>, val skipped: Int, val errors: List<String>)

private const val PROCESSED_FOLDER = "Processed"

/**
 * Walks [root] without following links. Skips `Processed` folders, [destinationRoot] and
 * hidden entries such as Android's `.trashed-*` and `.thumbnails`.
 */
fun scanMedia(root: Path, destinationRoot: Path): ScanResult {
    val media = mutableListOf<MediaFile>()
    val errors = mutableListOf<String>()
    var skipped = 0

    Files.walkFileTree(root, object : SimpleFileVisitor<Path>() {
        override fun preVisitDirectory(dir: Path, attrs: BasicFileAttributes): FileVisitResult {
            val skip = dir != root &&
                (isHidden(dir) || dir == destinationRoot || dir.name.equals(PROCESSED_FOLDER, ignoreCase = true))
            return if (skip) FileVisitResult.SKIP_SUBTREE else FileVisitResult.CONTINUE
        }

        override fun visitFile(file: Path, attrs: BasicFileAttributes): FileVisitResult {
            val kind = if (attrs.isRegularFile && !isHidden(file)) mediaKindOf(file.extension) else null
            if (kind == null) skipped++ else media += MediaFile(file, kind)
            return FileVisitResult.CONTINUE
        }

        override fun visitFileFailed(file: Path, exc: IOException): FileVisitResult {
            errors += "$file: ${exc.message}"
            return FileVisitResult.CONTINUE
        }
    })
    return ScanResult(media, skipped, errors)
}

fun isInsideProcessed(path: Path): Boolean =
    path.any { it.toString().equals(PROCESSED_FOLDER, ignoreCase = true) }

/**
 * Whether [scanMedia] of [root] holds [path]. Earlier days of a place outside the scan move with
 * it, as on desktop. The default destination `<root>/Processed` holds no scan files.
 */
fun isScanned(path: Path, root: Path, destinationRoot: Path): Boolean {
    if (!path.startsWith(root) || path == root) return false
    val inside = root.relativize(path)
    if (inside.any { it.toString().equals(PROCESSED_FOLDER, ignoreCase = true) || it.toString().startsWith(".") }) return false
    val destinationInside = destinationRoot != root && destinationRoot.startsWith(root)
    return !(destinationInside && path.startsWith(destinationRoot))
}

private fun isHidden(path: Path) = path.name.startsWith(".")
