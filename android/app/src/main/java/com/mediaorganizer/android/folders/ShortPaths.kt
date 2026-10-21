package com.mediaorganizer.android.folders

private const val HOME = "~"

/** Shows the internal storage root, such as `/storage/emulated/0`, as `~` so the field fits the folder name. */
fun shortPath(path: String, primaryRoot: String): String = when {
    path == primaryRoot -> HOME
    path.startsWith("$primaryRoot/") -> HOME + path.removePrefix(primaryRoot)
    else -> path
}

/** The reverse of [shortPath]. A typed full path stays as it is. */
fun fullPath(text: String, primaryRoot: String): String {
    val clean = text.trim()
    return when {
        clean == HOME -> primaryRoot
        clean.startsWith("$HOME/") -> primaryRoot + clean.removePrefix(HOME)
        else -> clean
    }
}
