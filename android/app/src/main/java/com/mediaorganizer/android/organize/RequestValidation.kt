package com.mediaorganizer.android.organize

import java.nio.file.Files
import java.nio.file.Path
import java.nio.file.Paths

sealed interface RequestCheck {
    data class Valid(val request: OrganizeRequest) : RequestCheck
    data class Invalid(val message: String) : RequestCheck
}

/** Checks the typed folders. A blank destination means `<source>/Processed`, as on desktop. */
fun checkRequest(sourceText: String, destinationText: String): RequestCheck {
    val source = Paths.get(sourceText.trim())
    if (!source.isAbsolute) return RequestCheck.Invalid("Gallery folder must be a full path such as ~/DCIM/Camera.")
    if (!Files.isDirectory(source)) return RequestCheck.Invalid("Gallery folder does not exist: $source")
    if (isInsideProcessed(source)) return RequestCheck.Invalid("Gallery folder must not be inside a Processed folder.")

    val destination = destinationText.trim().ifEmpty { null }?.let { Paths.get(it) } ?: source.resolve("Processed")
    if (!destination.isAbsolute) return RequestCheck.Invalid("Move to must be a full path.")
    if (destination == source) return RequestCheck.Invalid("Move to must differ from the gallery folder.")
    return RequestCheck.Valid(OrganizeRequest(source.normalize(), destination.normalize()))
}
