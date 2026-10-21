package com.mediaorganizer.android.folders

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

private const val ROOT = "/storage/emulated/0"

class StoragePathsTest {
    @Test fun internalStorageMapsToItsRoot() {
        assertEquals("$ROOT/DCIM/Camera", pathOfDocumentId("primary:DCIM/Camera", ROOT))
        assertEquals(ROOT, pathOfDocumentId("primary:", ROOT))
    }

    @Test fun memoryCardMapsUnderStorage() {
        assertEquals("/storage/1A2B-3C4D/Photos", pathOfDocumentId("1A2B-3C4D:Photos", ROOT))
    }

    @Test fun documentIdWithoutVolumeIsUnknown() {
        assertNull(pathOfDocumentId("downloads", ROOT))
    }

    @Test fun pathsMapBackToDocumentIds() {
        assertEquals("primary:DCIM/Camera", documentIdOfPath("$ROOT/DCIM/Camera/", ROOT))
        assertEquals("primary:", documentIdOfPath(ROOT, ROOT))
        assertEquals("1A2B-3C4D:Photos", documentIdOfPath("/storage/1A2B-3C4D/Photos", ROOT))
        assertNull(documentIdOfPath("/storage/emulated/1/DCIM", ROOT))
        assertNull(documentIdOfPath("/sdcard/DCIM", ROOT))
    }
}
