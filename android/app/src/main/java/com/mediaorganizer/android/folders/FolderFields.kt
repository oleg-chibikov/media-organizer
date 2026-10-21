package com.mediaorganizer.android.folders

import android.app.Activity
import android.content.Intent
import android.widget.Button
import android.widget.EditText
import android.widget.Toast
import com.mediaorganizer.android.R
import com.mediaorganizer.android.hasAllFilesAccessOrAsk

private const val PICK_SOURCE = 1
private const val PICK_DESTINATION = 2

/** The gallery and Move to fields with their Choose buttons, saved between launches. */
class FolderFields(
    private val activity: Activity,
    private val sourceField: EditText,
    private val destinationField: EditText,
    pickSource: Button,
    pickDestination: Button,
) {
    private val saved = SavedFolders(activity)

    val source: String get() = sourceField.text.toString()
    val destination: String get() = destinationField.text.toString()

    init {
        sourceField.setText(saved.source)
        destinationField.setText(saved.destination)
        pickSource.setOnClickListener { pick(PICK_SOURCE, source, createMissing = false) }
        // `Processed` often doesn't exist yet. Its parent is the gallery folder, an easy wrong pick.
        pickDestination.setOnClickListener { pick(PICK_DESTINATION, destination, createMissing = true) }
    }

    fun save() = saved.save(source, destination)

    /** Returns false for a result that belongs to another request. */
    fun onPicked(requestCode: Int, resultCode: Int, data: Intent?): Boolean {
        if (requestCode != PICK_SOURCE && requestCode != PICK_DESTINATION) return false
        if (resultCode != Activity.RESULT_OK || data == null) return true
        val path = FolderPicker.pathOf(data)
        if (path == null) {
            Toast.makeText(activity, R.string.pick_phone_folder, Toast.LENGTH_LONG).show()
            return true
        }
        if (requestCode == PICK_SOURCE) {
            if (destination.trim() == defaultDestination(source.trim())) destinationField.setText(defaultDestination(path))
            sourceField.setText(path)
        } else {
            destinationField.setText(path)
        }
        save()
        return true
    }

    private fun pick(requestCode: Int, currentPath: String, createMissing: Boolean) {
        if (!activity.hasAllFilesAccessOrAsk()) return
        activity.startActivityForResult(FolderPicker.intent(currentPath, createMissing), requestCode)
    }
}
