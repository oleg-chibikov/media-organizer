package com.mediaorganizer.android.geo

import java.io.BufferedInputStream
import java.io.BufferedReader
import java.io.InputStream
import java.util.zip.GZIPInputStream

private const val GZIP_MAGIC = 0x8b1f

/**
 * The lines of a place table. The repo keeps the tables gzipped, and the Android build unpacks
 * `.gz` assets into the APK, so both forms come in.
 */
internal fun tableReader(stream: InputStream): BufferedReader {
    val buffered = BufferedInputStream(stream)
    buffered.mark(2)
    val magic = buffered.read() or (buffered.read() shl 8)
    buffered.reset()
    val text = if (magic == GZIP_MAGIC) GZIPInputStream(buffered) else buffered
    return text.bufferedReader()
}
