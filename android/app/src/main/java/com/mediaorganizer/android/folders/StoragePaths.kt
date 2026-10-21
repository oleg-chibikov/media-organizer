package com.mediaorganizer.android.folders

private const val PRIMARY_VOLUME = "primary"
private val removableVolume = Regex("^/storage/([^/]+)(?:/(.*))?$")

/**
 * Turns a picker document id such as `primary:DCIM/Camera` into a file path.
 * The internal storage volume is called `primary`. A memory card goes by its id under `/storage`.
 */
fun pathOfDocumentId(documentId: String, primaryRoot: String): String? {
    if (!documentId.contains(':')) return null
    val volume = documentId.substringBefore(':')
    val relative = documentId.substringAfter(':').trim('/')
    val root = if (volume == PRIMARY_VOLUME) primaryRoot else "/storage/$volume"
    return if (relative.isEmpty()) root else "$root/$relative"
}

/** The reverse of [pathOfDocumentId], so the picker opens at the folder in the field. */
fun documentIdOfPath(path: String, primaryRoot: String): String? {
    val clean = path.trim().trimEnd('/')
    if (clean == primaryRoot || clean.startsWith("$primaryRoot/")) {
        return "$PRIMARY_VOLUME:" + clean.removePrefix(primaryRoot).trimStart('/')
    }
    val match = removableVolume.find(clean) ?: return null
    val volume = match.groupValues[1]
    if (volume == "emulated" || volume == "self") return null
    return "$volume:${match.groupValues[2]}"
}
