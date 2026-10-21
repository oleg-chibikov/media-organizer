package com.mediaorganizer.android

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Environment
import android.provider.Settings
import android.widget.Toast

/** Opens the system access screen when the app can't rename files yet. The person then presses the button again. */
fun Activity.hasAllFilesAccessOrAsk(): Boolean {
    if (Environment.isExternalStorageManager()) return true
    Toast.makeText(this, R.string.allow_all_files, Toast.LENGTH_LONG).show()
    startActivity(Intent(Settings.ACTION_MANAGE_APP_ALL_FILES_ACCESS_PERMISSION, Uri.parse("package:$packageName")))
    return false
}
