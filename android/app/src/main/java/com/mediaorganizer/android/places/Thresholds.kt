package com.mediaorganizer.android.places

import java.time.Duration

// Every number place detection decides by, as in `src-tauri/src/file_places/thresholds.rs`.

/** Home needs this many distinct days in one city. */
internal const val HOME_MIN_DAYS = 14
/** A home learned from a run and saved for later runs needs this many days, plus the same share. */
internal const val LEARNED_HOME_MIN_DAYS = 60
/** Home also needs this share, in percent, of all geotagged days. */
internal const val HOME_MIN_DAY_SHARE_PERCENT = 30
/** A file with no GPS joins the place of a geotagged file from the same device this close in time. */
internal val UNTAGGED_MAX_GAP: Duration = Duration.ofHours(3)
/** Points closer than this, in degrees, share one geocoder lookup. */
internal const val GEOCODE_CACHE_STEP_DEGREES = 0.001
