package com.mediaorganizer.android.location

import android.app.Activity
import android.app.AlertDialog
import android.os.Handler
import android.os.Looper
import android.text.Editable
import android.text.TextWatcher
import android.view.View
import android.widget.ArrayAdapter
import android.widget.EditText
import android.widget.ListView
import android.widget.TextView
import com.mediaorganizer.android.R
import com.mediaorganizer.android.geo.Place
import com.mediaorganizer.android.places.HomeLocation
import com.mediaorganizer.android.places.HomeSource
import com.mediaorganizer.android.places.homeAt
import kotlin.concurrent.thread

private const val SEARCH_DELAY_MILLIS = 200L
private const val MAX_CITIES = 10

/** Picks home by city name, as the desktop Home section does, or from the phone's location. */
class HomeDialog(
    private val activity: Activity,
    private val onPicked: (HomeLocation) -> Unit,
    onLocate: () -> Unit,
) {
    private val view = View.inflate(activity, R.layout.home_dialog, null)
    private val query: EditText = view.findViewById(R.id.city_query)
    private val status: TextView = view.findViewById(R.id.city_status)
    private val list: ListView = view.findViewById(R.id.city_list)
    private val adapter = ArrayAdapter<String>(activity, android.R.layout.simple_list_item_1)
    private val handler = Handler(Looper.getMainLooper())
    private val dialog = AlertDialog.Builder(activity)
        .setTitle(R.string.home_heading)
        .setView(view)
        .setNeutralButton(R.string.home_use_location) { _, _ -> onLocate() }
        .setNegativeButton(android.R.string.cancel, null)
        .create()
    private var cities: List<Place> = emptyList()
    // Drops answers to a query the person already changed.
    private var generation = 0

    init {
        list.adapter = adapter
        list.setOnItemClickListener { _, _, position, _ ->
            onPicked(homeAt(cities[position], HomeSource.MANUAL))
            dialog.dismiss()
        }
        query.addTextChangedListener(object : TextWatcher {
            override fun beforeTextChanged(text: CharSequence?, start: Int, count: Int, after: Int) = Unit
            override fun onTextChanged(text: CharSequence?, start: Int, before: Int, count: Int) = Unit
            override fun afterTextChanged(text: Editable?) = onQueryChanged(text.toString())
        })
        dialog.setOnDismissListener { handler.removeCallbacksAndMessages(null) }
    }

    fun show() = dialog.show()

    private fun onQueryChanged(text: String) {
        val current = ++generation
        handler.removeCallbacksAndMessages(null)
        showCities(emptyList())
        if (text.isBlank()) {
            status.visibility = View.GONE
            return
        }
        showStatus(R.string.home_searching)
        handler.postDelayed({ search(text, current) }, SEARCH_DELAY_MILLIS)
    }

    // The first search loads the place tables, too slow for the UI thread.
    private fun search(text: String, current: Int) = thread {
        val found = AppGeocoder.of(activity).searchCities(text, MAX_CITIES)
        activity.runOnUiThread {
            if (current != generation || !dialog.isShowing) return@runOnUiThread
            showCities(found)
            if (found.isEmpty()) showStatus(R.string.home_no_cities) else status.visibility = View.GONE
        }
    }

    private fun showStatus(text: Int) {
        status.setText(text)
        status.visibility = View.VISIBLE
    }

    private fun showCities(next: List<Place>) {
        cities = next
        adapter.clear()
        adapter.addAll(next.map { listOfNotNull(it.name, it.admin1, it.countryName).joinToString(", ") })
    }
}
