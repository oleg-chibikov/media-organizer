package com.mediaorganizer.android.folders

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Rule
import org.junit.Test
import org.junit.rules.TemporaryFolder
import java.io.File

class FolderTreeTest {
    @get:Rule val temp = TemporaryFolder()

    @Test fun subfoldersSkipFilesAndHiddenFolders() {
        val root = temp.newFolder("phone")
        listOf("camera", "Alpha", ".thumbnails").forEach { File(root, it).mkdir() }
        File(root, "photo.jpg").createNewFile()
        assertEquals(listOf("Alpha", "camera"), subfolders(root).map(File::getName))
    }

    @Test fun startFolderIsTheClosestExistingFolder() {
        val root = temp.newFolder("phone")
        val camera = File(root, "DCIM/Camera").apply { mkdirs() }
        assertEquals(camera, startFolder("${camera.path}/Processed/2024", listOf(root)))
        assertEquals(camera, startFolder(camera.path, listOf(root)))
    }

    @Test fun startFolderIsNullOutsideStorage() {
        val root = temp.newFolder("phone")
        assertNull(startFolder("/elsewhere/DCIM", listOf(root)))
        assertNull(startFolder("DCIM", listOf(root)))
        assertNull(startFolder("${root.path}-other/DCIM", listOf(root)))
    }

    @Test fun upStopsAtStorageRoot() {
        val root = temp.newFolder("phone")
        val dcim = File(root, "DCIM")
        assertEquals(root, folderAbove(dcim, listOf(root)))
        assertNull(folderAbove(root, listOf(root)))
    }
}
