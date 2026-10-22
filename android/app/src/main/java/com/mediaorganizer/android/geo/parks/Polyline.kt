package com.mediaorganizer.android.geo.parks

private const val SCALE = 1e5

/** Decodes a Google polyline string into interleaved `lat, lon` points, as `park_lines.py` writes them. */
internal fun decodePolyline(encoded: String): FloatArray? {
    val reader = NumberReader(encoded)
    var points = FloatArray(maxOf(8, encoded.length / 2))
    var count = 0
    var latitude = 0L
    var longitude = 0L
    while (!reader.done) {
        latitude += reader.next() ?: return null
        longitude += reader.next() ?: return null
        if (count + 2 > points.size) points = points.copyOf(points.size * 2)
        points[count++] = (latitude / SCALE).toFloat()
        points[count++] = (longitude / SCALE).toFloat()
    }
    return points.copyOf(count)
}

private class NumberReader(private val encoded: String) {
    private var at = 0

    val done: Boolean get() = at >= encoded.length

    fun next(): Long? {
        var value = 0L
        var shift = 0
        while (true) {
            if (at >= encoded.length) return null
            val chunk = encoded[at++].code - 63
            if (chunk < 0) return null
            value = value or ((chunk.toLong() and 0x1F) shl shift)
            shift += 5
            if (chunk < 0x20) break
        }
        return if (value and 1L == 1L) (value shr 1).inv() else value shr 1
    }
}
