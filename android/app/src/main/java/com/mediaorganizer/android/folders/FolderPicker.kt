package com.mediaorganizer.android.folders

import android.content.Intent
import android.os.Environment
import android.provider.DocumentsContract
import java.io.File

private const val EXTERNAL_STORAGE = "com.android.externalstorage.documents"

/** Opens the system folder picker and reads the chosen folder back as a file path. */
object FolderPicker {
    fun intent(currentPath: String): Intent = Intent(Intent.ACTION_OPEN_DOCUMENT_TREE).apply {
        val start = nearestFolder(currentPath) ?: return@apply
        val documentId = documentIdOfPath(start, primaryRoot()) ?: return@apply
        putExtra(DocumentsContract.EXTRA_INITIAL_URI, DocumentsContract.buildDocumentUri(EXTERNAL_STORAGE, documentId))
    }

    /** Null for a folder outside the phone storage, such as a cloud drive. */
    fun pathOf(result: Intent): String? {
        val tree = result.data ?: return null
        if (tree.authority != EXTERNAL_STORAGE) return null
        return pathOfDocumentId(DocumentsContract.getTreeDocumentId(tree), primaryRoot())
    }

    /** `Processed` may not exist yet, so the picker opens at its parent. */
    private fun nearestFolder(path: String): String? =
        path.trim().takeIf { it.startsWith("/") }
            ?.let { generateSequence(File(it)) { file -> file.parentFile }.firstOrNull(File::isDirectory)?.path }

    private fun primaryRoot(): String = Environment.getExternalStorageDirectory().path
}
