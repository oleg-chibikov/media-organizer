package com.mediaorganizer.android.folders

import java.io.File

/** Visible subfolders, sorted by name the way a file manager shows them. */
fun subfolders(folder: File): List<File> =
    folder.listFiles { file -> file.isDirectory && !isHidden(file) }.orEmpty().sortedBy { it.name.lowercase() }

/** A typed path may not exist, so the picker opens at the closest folder above it. Null outside every storage. */
fun startFolder(path: String, roots: List<File>): File? {
    if (!File(path).isAbsolute) return null
    val typed = File(path)
    val root = roots.firstOrNull { isInside(typed, it) } ?: return null
    return generateSequence(typed) { it.parentFile }.takeWhile { isInside(it, root) }.firstOrNull(File::isDirectory)
}

/** Up stops at a storage root. Null there stands for the list of storages. */
fun folderAbove(folder: File, roots: List<File>): File? = if (folder in roots) null else folder.parentFile

// A dot marks a hidden folder on Android. File.isHidden reads a file attribute on Windows, where the unit tests also run.
private fun isHidden(file: File) = file.name.startsWith(".")

private fun isInside(file: File, root: File) = file.toPath().startsWith(root.toPath())
