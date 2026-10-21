use super::{timestamp::parse_metadata_timestamp, RawMetadataDates, SelectedMetadataDate};
use crate::models::FileKind;

const FALLBACK_TAG: &str = "FileModifyDate";

/// Picks the first parseable date in the file kind's tag priority order.
pub(crate) fn select_metadata_date(
    file_kind: &FileKind,
    raw_dates: &RawMetadataDates,
) -> Result<Option<SelectedMetadataDate>, String> {
    let mut parse_failures: Vec<String> = Vec::new();

    for (tag, value) in metadata_priority(file_kind, raw_dates) {
        let Some(raw_value) = value else {
            continue;
        };

        if let Some((chosen_date, date_kind)) =
            parse_metadata_timestamp(raw_value, tag == FALLBACK_TAG)
        {
            return Ok(Some(SelectedMetadataDate {
                chosen_date,
                chosen_date_source: tag.to_string(),
                date_kind,
                raw_metadata_date: raw_value.to_string(),
            }));
        }

        parse_failures.push(format!("{tag}={raw_value}"));
    }

    if parse_failures.is_empty() {
        Ok(None)
    } else {
        Err(format!(
            "Could not parse metadata date values in priority order: {}",
            parse_failures.join(", ")
        ))
    }
}

fn metadata_priority<'a>(
    file_kind: &FileKind,
    raw_dates: &'a RawMetadataDates,
) -> [(&'static str, Option<&'a String>); 5] {
    match file_kind {
        FileKind::Photo => [
            (
                "SubSecDateTimeOriginal",
                raw_dates.sub_sec_date_time_original.as_ref(),
            ),
            ("DateTimeOriginal", raw_dates.date_time_original.as_ref()),
            ("CreateDate", raw_dates.create_date.as_ref()),
            ("ModifyDate", raw_dates.modify_date.as_ref()),
            (FALLBACK_TAG, raw_dates.file_modify_date.as_ref()),
        ],
        FileKind::Video => [
            ("MediaCreateDate", raw_dates.media_create_date.as_ref()),
            ("TrackCreateDate", raw_dates.track_create_date.as_ref()),
            ("CreateDate", raw_dates.create_date.as_ref()),
            ("ModifyDate", raw_dates.modify_date.as_ref()),
            (FALLBACK_TAG, raw_dates.file_modify_date.as_ref()),
        ],
        _ => [
            ("CreateDate", raw_dates.create_date.as_ref()),
            ("ModifyDate", raw_dates.modify_date.as_ref()),
            (FALLBACK_TAG, raw_dates.file_modify_date.as_ref()),
            ("DateTimeOriginal", raw_dates.date_time_original.as_ref()),
            ("MediaCreateDate", raw_dates.media_create_date.as_ref()),
        ],
    }
}
