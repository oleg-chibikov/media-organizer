package com.mediaorganizer.android.folders

import android.app.Activity
import android.app.AlertDialog
import android.content.DialogInterface
import android.widget.ArrayAdapter
import android.widget.ListView
import com.mediaorganizer.android.R
import java.io.File
import kotlin.concurrent.thread

/**
 * Picks a folder inside the app. Back closes it and returns to the main screen.
 * The system picker walked up the folders on Back instead.
 */
class FolderDialog(private val activity: Activity, private val onChosen: (String) -> Unit) {
    private class Row(val label: String, val target: File?)

    private val roots = storageRoots(activity)
    private val rootFolders = roots.map(StorageRoot::folder)
    private val home = primaryRoot()
    private val adapter = ArrayAdapter<String>(activity, android.R.layout.simple_list_item_1)
    private val list = ListView(activity).apply { adapter = this@FolderDialog.adapter }
    private val dialog = AlertDialog.Builder(activity)
        .setTitle(R.string.storages)
        .setView(list)
        .setPositiveButton(R.string.use_folder) { _, _ -> folder?.let { onChosen(it.path) } }
        .setNegativeButton(android.R.string.cancel, null)
        .create()
    private var folder: File? = null
    private var rows: List<Row> = emptyList()

    init {
        list.setOnItemClickListener { _, _, position, _ -> open(rows[position].target) }
    }

    fun show(currentPath: String) {
        dialog.show()
        open(startFolder(currentPath, rootFolders) ?: rootFolders.singleOrNull())
    }

    private fun open(next: File?) {
        folder = next
        dialog.getButton(DialogInterface.BUTTON_POSITIVE).isEnabled = next != null
        if (next == null) {
            dialog.setTitle(R.string.storages)
            showRows(roots.map { Row(it.label, it.folder) })
            return
        }
        dialog.setTitle(shortPath(next.path, home))
        val canGoUp = next !in rootFolders || roots.size > 1
        val up = if (canGoUp) listOf(Row(activity.getString(R.string.up), folderAbove(next, rootFolders))) else emptyList()
        showRows(up)
        // A camera folder holds thousands of files, too slow to list on the UI thread.
        thread {
            val children = subfolders(next).map { Row(it.name, it) }
            activity.runOnUiThread { if (folder == next && dialog.isShowing) showRows(up + children) }
        }
    }

    private fun showRows(next: List<Row>) {
        rows = next
        adapter.clear()
        adapter.addAll(next.map(Row::label))
        list.setSelection(0)
    }
}
