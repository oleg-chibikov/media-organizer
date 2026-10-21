package com.mediaorganizer.android.metadata

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class MetadataFormatTest {
    @Test fun retrieverDateBecomesExifToolDate() =
        assertEquals("2024:05:01 10:15:00", retrieverDateToExifTool("20240501T101500.000Z"))

    @Test fun emptyOrBadRetrieverDateIsMissing() {
        assertNull(retrieverDateToExifTool(null))
        assertNull(retrieverDateToExifTool("19040101T000000.000Z"))
        assertNull(retrieverDateToExifTool("garbage"))
    }

    @Test fun subSecDateJoinsLikeExifTool() {
        assertEquals("2024:05:01 10:00:00.123+10:00", subSecDate("2024:05:01 10:00:00", "123", "+10:00"))
        assertEquals("2024:05:01 10:00:00", subSecDate("2024:05:01 10:00:00", null, null))
    }
}
