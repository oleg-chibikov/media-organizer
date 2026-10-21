package com.mediaorganizer.android.organize

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.time.LocalDateTime
import java.time.ZoneId

class CaptureDateTest {
    private val sydney = ZoneId.of("Australia/Sydney")

    private fun found(kind: MediaKind, raw: RawMetadata) = selectCaptureDate(kind, raw, sydney) as CaptureDate.Found

    @Test fun photoPrefersSubSecDateOverOtherTags() {
        val raw = RawMetadata(
            subSecDateTimeOriginal = "2024:05:01 10:00:00.123+10:00",
            dateTimeOriginal = "2024:05:01 09:00:00",
            fileModifyDate = "2025:01:01 00:00:00+11:00",
        )
        assertEquals("SubSecDateTimeOriginal", found(MediaKind.PHOTO, raw).sourceTag)
    }

    @Test fun zonedDateConvertsToLocalTime() {
        val raw = RawMetadata(subSecDateTimeOriginal = "2024:05:01 00:30:00+00:00")
        assertEquals(LocalDateTime.of(2024, 5, 1, 10, 30), found(MediaKind.PHOTO, raw).date)
    }

    @Test fun videoSkipsPhotoTagsAndFallsBackToFileDate() {
        val raw = RawMetadata(dateTimeOriginal = "2024:05:01 09:00:00", fileModifyDate = "2025:01:02 03:04:05+11:00")
        val date = found(MediaKind.VIDEO, raw)
        assertEquals("FileModifyDate", date.sourceTag)
        assertEquals(LocalDateTime.of(2025, 1, 2, 3, 4, 5), date.date)
    }

    @Test fun unparseableValueFallsThroughToNextTag() {
        val raw = RawMetadata(dateTimeOriginal = "0000:00:00 00:00:00", createDate = "2024:02:29 12:00:00")
        assertEquals(LocalDateTime.of(2024, 2, 29, 12, 0), found(MediaKind.PHOTO, raw).date)
    }

    @Test fun onlyBadValuesAreUnparseable() {
        val result = selectCaptureDate(MediaKind.PHOTO, RawMetadata(modifyDate = "2023:02:30 12:00:00"), sydney)
        assertTrue(result is CaptureDate.Unparseable)
    }

    @Test fun noValuesIsMissing() =
        assertEquals(CaptureDate.Missing, selectCaptureDate(MediaKind.PHOTO, RawMetadata(), sydney))

    @Test fun localTimeInDaylightSavingGapIsRejected() =
        assertNull(parseMetadataTimestamp("2024:10:06 02:30:00", sydney))
}
