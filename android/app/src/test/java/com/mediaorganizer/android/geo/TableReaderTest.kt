package com.mediaorganizer.android.geo

import org.junit.Assert.assertEquals
import org.junit.Test
import java.io.ByteArrayOutputStream
import java.util.zip.GZIPOutputStream

class TableReaderTest {
    private val text = "C\tJP\tJapan\nA\tKyoto\n"

    @Test fun readsPlainTables() = assertEquals(text.lines(), tableReader(text.byteInputStream()).readText().lines())

    @Test fun readsGzippedTables() {
        val gzipped = ByteArrayOutputStream().also { out -> GZIPOutputStream(out).use { it.write(text.toByteArray()) } }
        assertEquals(text.lines(), tableReader(gzipped.toByteArray().inputStream()).readText().lines())
    }
}
