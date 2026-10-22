package com.mediaorganizer.android.metadata

import com.mediaorganizer.android.geo.Coordinates
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class GpsPointTest {
    @Test fun readsTheVideoLocation() {
        assertEquals(Coordinates(37.7749, -122.4194), parseIso6709("+37.7749-122.4194+010.000/"))
        assertEquals(Coordinates(-33.8568, 151.2153), parseIso6709("-33.8568+151.2153/"))
    }

    @Test fun dropsNullIslandAndBadValues() {
        assertNull(parseIso6709("+00.0000+000.0000/"))
        assertNull(parseIso6709("37.7749-122.4194"))
        assertNull(parseIso6709(null))
        assertNull(gpsPoint(91.0, 10.0))
    }
}
