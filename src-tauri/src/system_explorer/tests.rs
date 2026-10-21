#[cfg(target_os = "windows")]
mod windows {
    use std::path::Path;

    use crate::system_explorer::strip_windows_verbatim_prefix;

    #[test]
    fn strips_verbatim_disk_prefix_for_explorer() {
        assert_eq!(
            strip_windows_verbatim_prefix(Path::new(r"\\?\C:\todo\IMG 1001.jpg")),
            Path::new(r"C:\todo\IMG 1001.jpg")
        );
    }

    #[test]
    fn strips_verbatim_unc_prefix_for_explorer() {
        assert_eq!(
            strip_windows_verbatim_prefix(Path::new(r"\\?\UNC\nas\photos\TODO\IMG 1001.jpg")),
            Path::new(r"\\nas\photos\TODO\IMG 1001.jpg")
        );
    }
}
