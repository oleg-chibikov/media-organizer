use std::path::Path;

use super::long_path_form;

#[test]
fn prefixes_long_disk_and_unc_paths_only() {
    let long_name = "a".repeat(260);
    let disk = format!(r"C:\media\{long_name}.jpg");
    let unc = format!(r"\\nas\share\{long_name}.jpg");

    assert_eq!(
        long_path_form(Path::new(r"C:\a.jpg")),
        Path::new(r"C:\a.jpg")
    );
    assert_eq!(
        long_path_form(Path::new(&disk)),
        Path::new(&format!(r"\\?\{disk}"))
    );
    assert_eq!(
        long_path_form(Path::new(&unc)),
        Path::new(&format!(r"\\?\UNC\nas\share\{long_name}.jpg"))
    );
}
