package com.mediaorganizer.android.geo

import com.mediaorganizer.android.geo.parks.decodePolyline
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertNull
import org.junit.Test

class PolylineTest {
    @Test fun decodesTheReferenceExample() {
        val points = decodePolyline("_p~iF~ps|U_ulLnnqC_mqNvxq`@")
        assertArrayEquals(floatArrayOf(38.5f, -120.2f, 40.7f, -120.95f, 43.252f, -126.453f), points, 0f)
    }

    @Test fun cutOffNumberIsInvalid() = assertNull(decodePolyline("_p~iF~ps|"))
}
