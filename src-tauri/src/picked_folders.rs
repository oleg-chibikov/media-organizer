//! Folders the user chose in the native picker this session. Moves start only from one of
//! them, so a webview request can't aim a move at a folder the user never selected.

use std::{
    collections::HashSet,
    path::Path,
    sync::{Mutex, PoisonError},
};

use crate::path_safety::normalized_path_string;

#[derive(Default)]
pub(crate) struct PickedFolders(Mutex<HashSet<String>>);

impl PickedFolders {
    pub(crate) fn remember(&self, folder: &Path) {
        self.0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .insert(normalized_path_string(folder));
    }

    pub(crate) fn ensure_picked(&self, folder: &Path) -> Result<(), String> {
        let picked = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if picked.contains(&normalized_path_string(folder)) {
            Ok(())
        } else {
            Err("Moves can only start from a folder chosen in the folder picker.".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::PickedFolders;

    #[test]
    fn accepts_only_folders_the_user_picked() {
        let picked = PickedFolders::default();
        picked.remember(Path::new("/media/TODO"));

        assert!(picked.ensure_picked(Path::new("/media/TODO/")).is_ok());
        assert!(picked.ensure_picked(Path::new("/media")).is_err());
        assert!(picked.ensure_picked(Path::new("/")).is_err());
    }
}
