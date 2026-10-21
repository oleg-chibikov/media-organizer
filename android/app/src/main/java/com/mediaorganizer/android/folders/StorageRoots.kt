package com.mediaorganizer.android.folders

import android.content.Context
import android.os.Environment
import android.os.storage.StorageManager
import java.io.File

/** Where the folder picker starts: the phone storage or a memory card. */
class StorageRoot(val folder: File, val label: String)

fun primaryRoot(): String = Environment.getExternalStorageDirectory().path

fun storageRoots(context: Context): List<StorageRoot> =
    context.getSystemService(StorageManager::class.java).storageVolumes
        .filter { it.state == Environment.MEDIA_MOUNTED }
        .mapNotNull { volume -> volume.directory?.let { StorageRoot(it, volume.getDescription(context)) } }
