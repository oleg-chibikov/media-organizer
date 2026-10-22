package com.mediaorganizer.android.geo

import java.nio.file.Files
import java.nio.file.Path

/** The desktop app's gzipped place tables, loaded once for every test that needs real places. */
object TestGeocoder {
    private val assets: Path = Path.of("../../src-tauri/assets")

    val geocoder: Geocoder by lazy { Geocoder { name -> Files.newInputStream(assets.resolve("$name.gz")) } }
}
