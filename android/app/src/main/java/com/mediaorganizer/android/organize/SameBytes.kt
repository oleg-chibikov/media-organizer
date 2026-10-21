package com.mediaorganizer.android.organize

import java.io.InputStream
import java.nio.file.Files
import java.nio.file.Path

private const val COMPARE_BUFFER_BYTES = 64 * 1024

/** True when both files have the same size and bytes. */
fun filesHaveSameBytes(left: Path, right: Path): Boolean {
    if (Files.size(left) != Files.size(right)) return false
    val leftBuffer = ByteArray(COMPARE_BUFFER_BYTES)
    val rightBuffer = ByteArray(COMPARE_BUFFER_BYTES)

    Files.newInputStream(left).use { leftStream ->
        Files.newInputStream(right).use { rightStream ->
            while (true) {
                val read = leftStream.fill(leftBuffer)
                if (rightStream.fill(rightBuffer) != read) return false
                for (index in 0 until read) {
                    if (leftBuffer[index] != rightBuffer[index]) return false
                }
                if (read == 0) return true
            }
        }
    }
}

private fun InputStream.fill(buffer: ByteArray): Int {
    var filled = 0
    while (filled < buffer.size) {
        val read = read(buffer, filled, buffer.size - filled)
        if (read < 0) break
        filled += read
    }
    return filled
}
