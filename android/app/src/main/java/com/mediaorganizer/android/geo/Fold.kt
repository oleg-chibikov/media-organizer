package com.mediaorganizer.android.geo

import java.util.Locale

/**
 * Folds names so "Zür", "Zur" and "Zuerich" all match the asset's "Zuerich". Lower-cases, strips
 * accents from Latin letters and drops the `e` of the German `ae`, `oe` and `ue` spellings.
 */
internal fun fold(text: String): String {
    val folded = StringBuilder(text.length)
    for (letter in text.lowercase(Locale.ROOT)) folded.append(baseLetters[letter] ?: letter.toString())
    return folded.toString().replace("ae", "a").replace("oe", "o").replace("ue", "u")
}

private val baseLetters: Map<Char, String> = buildMap {
    fun put(letters: String, base: String) = letters.forEach { put(it, base) }
    put("àáâãäåāăą", "a")
    put("æ", "ae")
    put("çćĉċč", "c")
    put("ďđð", "d")
    put("èéêëēĕėęě", "e")
    put("ĝğġģ", "g")
    put("ĥħ", "h")
    put("ìíîïĩīĭįı", "i")
    put("ĵ", "j")
    put("ķ", "k")
    put("ĺļľŀł", "l")
    put("ñńņň", "n")
    put("òóôõöøōŏő", "o")
    put("œ", "oe")
    put("ŕŗř", "r")
    put("śŝşšș", "s")
    put("ß", "ss")
    put("ţťŧț", "t")
    put("þ", "th")
    put("ùúûüũūŭůűų", "u")
    put("ŵ", "w")
    put("ýÿŷ", "y")
    put("źżž", "z")
}
