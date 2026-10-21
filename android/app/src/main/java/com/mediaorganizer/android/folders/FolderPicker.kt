package com.mediaorganizer.android.folders

import android.content.Intent
import android.os.Environment
import android.provider.DocumentsContract
import java.io.File

private const val EXTERNAL_STORAGE = "com.android.externalstorage.documents"

/** Opens the system folder picker and reads the chosen folder back as a file path. */
object FolderPicker {
    fun intent(currentPath: String, createMissing: Boolean): Intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
        val path = currentPath.trim().takeIf { it.startsWith("/") } ?: return@apply
        if (createMissing) File(path).mkdirs()
        val start = nearestFolder(path) ?: return@apply
        val documentId = documentIdOfPath(start, primaryRoot()) ?: return@apply
        putExtra(DocumentsContract.EXTRA_INITIAL_URI, DocumentsContract.buildDocumentUri(EXTERNAL_STORAGE, documentId))
    }

    /** Null for a folder outside the phone storage, such as a cloud drive. */
    fun pathOf(result: Intent): String? {
        val tree = result.data ?: return null
        if (tree.authority != EXTERNAL_STORAGE) return null
        return pathOfDocumentId(DocumentsContract.getTreeDocumentId(tree), primaryRoot())
    }

    /** A typed path may not exist, so the picker opens at the closest folder above it. */
    private fun nearestFolder(path: String): String? =
        generateSequence(File(path)) { file -> file.parentFile }.firstOrNull(File::isDirectory)?.path

    private fun primaryRoot(): String = Environment.getExternalStorageDirectory().path
}
