package com.mediaorganizer.android.location

import android.content.Context
import android.widget.CompoundButton

/** The Group by place box: on by default, kept between launches, read by background runs too. */
object GroupByPlace {
    private const val KEY = "group_by_place"

    fun isOn(context: Context): Boolean = preferences(context).getBoolean(KEY, true)

    fun bind(box: CompoundButton) {
        box.isChecked = isOn(box.context)
        box.setOnCheckedChangeListener { _, checked -> preferences(box.context).edit().putBoolean(KEY, checked).apply() }
    }

    private fun preferences(context: Context) = context.getSharedPreferences("places", Context.MODE_PRIVATE)
}
