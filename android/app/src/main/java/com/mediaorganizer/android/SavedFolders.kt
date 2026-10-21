package com.mediaorganizer.android

import android.content.Context
import android.os.Environment
import java.io.File

/** The typed folders, kept between launches. The gallery folder starts as `DCIM/Camera`. */
class SavedFolders(context: Context) {
    private val preferences = context.getSharedPreferences("folders", Context.MODE_PRIVATE)

    val source: String
        get() = preferences.getString(SOURCE_KEY, null) ?: defaultGalleryFolder()

    val destination: String
        get() = preferences.getString(DESTINATION_KEY, null).orEmpty()

    fun save(source: String, destination: String) {
        preferences.edit().putString(SOURCE_KEY, source).putString(DESTINATION_KEY, destination).apply()
    }

    private fun defaultGalleryFolder(): String =
        File(Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DCIM), "Camera").path

    private companion object {
        const val SOURCE_KEY = "source"
        const val DESTINATION_KEY = "destination"
    }
}
