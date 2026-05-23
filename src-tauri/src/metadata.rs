#[derive(Debug, Clone)]
pub struct RawMetadataDates {
    pub sub_sec_date_time_original: Option<String>,
    pub date_time_original: Option<String>,
    pub create_date: Option<String>,
    pub modify_date: Option<String>,
    pub media_create_date: Option<String>,
    pub track_create_date: Option<String>,
    pub file_modify_date: Option<String>,
}

pub fn parse_exiftool_json(_json: &str) -> Result<RawMetadataDates, String> {
    Err("Metadata parser is not implemented yet.".to_string())
}
