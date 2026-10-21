package com.mediaorganizer.android.organize

import java.io.FileOutputStream
import java.io.IOException
import java.io.InputStream
import java.nio.file.FileAlreadyExistsException
import java.nio.file.Files
import java.nio.file.Path
import java.util.zip.CRC32

private const val COPY_BUFFER_BYTES = 256 * 1024
private const val MAX_TEMP_NAME_ATTEMPTS = 1000

/**
 * Copies [source] into a hidden file beside [destination], syncs it and reads it back once.
 * The copy then takes the destination name, and the source goes only if no app changed it meanwhile.
 */
internal fun copyThenRemoveSource(source: Path, destination: Path, rename: Rename) {
    val before = FileState.of(source)
    val temp = reserveTempBeside(destination)
    try {
        val copied = copySynced(source, temp)
        Files.setLastModifiedTime(temp, before.modified)
        if (checksumOf(temp) != copied) throw IOException("The copy of ${source.fileName} differs from it")
        requireUnchanged(source, before)
        rename.move(temp, destination)
    } catch (error: Exception) {
        Files.deleteIfExists(temp)
        throw error
    }
    requireUnchanged(source, before)
    Files.delete(source)
}

private fun requireUnchanged(source: Path, before: FileState) {
    if (FileState.of(source) != before) throw IOException("${source.fileName} changed while copying, so it stays")
}

/** Returns the checksum of the bytes written, so the source needs no second read. */
private fun copySynced(source: Path, target: Path): Long =
    Files.newInputStream(source).use { input ->
        FileOutputStream(target.toFile()).use { output ->
            val checksum = input.forEachChunk { buffer, read -> output.write(buffer, 0, read) }
            output.fd.sync()
            checksum
        }
    }

private fun checksumOf(path: Path): Long = Files.newInputStream(path).use { it.forEachChunk { _, _ -> } }

private inline fun InputStream.forEachChunk(onChunk: (ByteArray, Int) -> Unit): Long {
    val checksum = CRC32()
    val buffer = ByteArray(COPY_BUFFER_BYTES)
    while (true) {
        val read = read(buffer)
        if (read < 0) return checksum.value
        checksum.update(buffer, 0, read)
        onChunk(buffer, read)
    }
}

private fun tempName(destination: Path, attempt: Int) = ".${destination.fileName}.media-organizer-copying-$attempt.tmp"

/** Runs go one at a time, so a temp copy found here is left from a run the system killed. */
private fun removeOrphanedTemps(destination: Path) {
    val prefix = tempName(destination, 0).removeSuffix("0.tmp")
    val orphan = Regex("${Regex.escape(prefix)}\\d+\\.tmp")
    Files.newDirectoryStream(destination.parent) { orphan.matches(it.fileName.toString()) }.use { temps ->
        temps.forEach(Files::deleteIfExists)
    }
}

private fun reserveTempBeside(destination: Path): Path {
    removeOrphanedTemps(destination)
    for (attempt in 0 until MAX_TEMP_NAME_ATTEMPTS) {
        val temp = destination.resolveSibling(tempName(destination, attempt))
        try {
            return Files.createFile(temp)
        } catch (_: FileAlreadyExistsException) {
            continue
        }
    }
    throw IOException("No free name for a temporary copy beside $destination")
}
