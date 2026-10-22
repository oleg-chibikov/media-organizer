package com.mediaorganizer.android.geo

/** One place of the asset before sorting. Its name sits in the shared names text. */
internal class GeoRow(
    val latitude: Float,
    val longitude: Float,
    val kind: PlaceKind,
    val country: Int,
    val admin1: Int,
    val population: Int,
    val nameStart: Int,
    val nameEnd: Int,
)

/** An entry within a search circle, with its distance in km. */
internal class Hit(val entry: Int, val distance: Double)

/** The in-memory place table. Kept in flat arrays: names live in one shared text. */
internal class GeoIndex(
    rows: List<GeoRow>,
    private val names: String,
    private val countries: List<Pair<String, String>>,
    private val admin1Names: List<String>,
) {
    val size = rows.size
    private val latitudes = FloatArray(size)
    private val longitudes = FloatArray(size)
    private val kinds = arrayOfNulls<PlaceKind>(size)
    private val countryIds = IntArray(size)
    private val admin1Ids = IntArray(size)
    private val populations = IntArray(size)
    private val nameStarts = IntArray(size)
    private val nameEnds = IntArray(size)
    private val grid: Grid

    init {
        val cells = IntArray(size) { cellOf(rows[it].latitude.toDouble(), rows[it].longitude.toDouble()) }
        val order = rows.indices.sortedBy { cells[it] }
        order.forEachIndexed { entry, row ->
            val source = rows[row]
            latitudes[entry] = source.latitude
            longitudes[entry] = source.longitude
            kinds[entry] = source.kind
            countryIds[entry] = source.country
            admin1Ids[entry] = source.admin1
            populations[entry] = source.population
            nameStarts[entry] = source.nameStart
            nameEnds[entry] = source.nameEnd
        }
        grid = Grid.ofSortedCells(IntArray(size) { cells[order[it]] })
    }

    fun kind(entry: Int): PlaceKind = kinds[entry]!!

    fun population(entry: Int): Int = populations[entry]

    fun name(entry: Int): String = names.substring(nameStarts[entry], nameEnds[entry])

    fun coordinates(entry: Int) = Coordinates(latitudes[entry].toDouble(), longitudes[entry].toDouble())

    /** Entries within [radiusKm] of [point], with their distance. */
    fun near(point: Coordinates, radiusKm: Double): List<Hit> {
        val hits = mutableListOf<Hit>()
        grid.forEachRangeNear(point, radiusKm) { start, end ->
            for (entry in start until end) {
                val distance = distanceKm(point, coordinates(entry))
                if (distance <= radiusKm) hits += Hit(entry, distance)
            }
        }
        return hits
    }

    /** The nearest entry of any kind within [radiusKm]. */
    fun nearest(point: Coordinates, radiusKm: Double): Int? = near(point, radiusKm).minByOrNull(Hit::distance)?.entry

    fun place(entry: Int): Place {
        val (countryCode, countryName) = countries[countryIds[entry]]
        return Place(
            name = name(entry),
            kind = kind(entry),
            admin1 = admin1Names.getOrNull(admin1Ids[entry]),
            countryCode = countryCode,
            countryName = countryName,
            latitude = latitudes[entry].toDouble(),
            longitude = longitudes[entry].toDouble(),
            population = populations[entry],
        )
    }
}
