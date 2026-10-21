package com.mediaorganizer.android.organize

import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test
import java.nio.file.attribute.FileTime
import java.time.Instant

class FileStateTest {
    private val now = Instant.parse("2026-10-21T12:00:00Z")

    private fun changedAt(time: Instant) = FileState(1, FileTime.from(time))

    @Test fun fileChangedJustNowIsFresh() {
        assertTrue(changedAt(now.minusSeconds(5)).isFresh(now))
    }

    @Test fun fileQuietForTheSettleTimeIsSettled() {
        assertFalse(changedAt(now.minus(SETTLE_TIME)).isFresh(now))
    }

    @Test fun fileFromTheFutureIsSettled() {
        assertFalse(changedAt(now.plusSeconds(3600)).isFresh(now))
    }
}
