package com.mediaorganizer.android.folders

import android.app.Activity
import android.widget.Button
import android.widget.EditText
import com.mediaorganizer.android.hasAllFilesAccessOrAsk
import java.io.File

/** The gallery and Move to fields with their Choose buttons, saved between launches. */
class FolderFields(
    private val activity: Activity,
    private val sourceField: EditText,
    private val destinationField: EditText,
    pickSource: Button,
    pickDestination: Button,
) {
    private val saved = SavedFolders(activity)
    private val root = primaryRoot()

    val source: String get() = fullPath(sourceField.text.toString(), root)
    val destination: String get() = fullPath(destinationField.text.toString(), root)

    init {
        show(sourceField, saved.source)
        show(destinationField, saved.destination)
        pickSource.setOnClickListener { pick(source, createMissing = false, ::onSourcePicked) }
        // `Processed` often doesn't exist yet. Its parent is the gallery folder, an easy wrong pick.
        pickDestination.setOnClickListener { pick(destination, createMissing = true, ::onDestinationPicked) }
    }

    fun save() = saved.save(source, destination)

    private fun onSourcePicked(path: String) {
        if (destination == defaultDestination(source)) show(destinationField, defaultDestination(path))
        show(sourceField, path)
        save()
    }

    private fun onDestinationPicked(path: String) {
        show(destinationField, path)
        save()
    }

    private fun show(field: EditText, path: String) = field.setText(shortPath(path, root))

    private fun pick(currentPath: String, createMissing: Boolean, onChosen: (String) -> Unit) {
        if (!activity.hasAllFilesAccessOrAsk()) return
        if (createMissing && currentPath.startsWith("/")) File(currentPath).mkdirs()
        FolderDialog(activity, onChosen).show(currentPath)
    }
}
