use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// Checks folders under `root` once each, and skips a folder whose parent is missing.
/// A search over every day of a month then checks each missing day folder once.
pub(super) struct DirProbe<'a> {
    root: &'a Path,
    known: HashMap<PathBuf, bool>,
}

impl<'a> DirProbe<'a> {
    pub(super) fn new(root: &'a Path) -> Self {
        Self {
            root,
            known: HashMap::new(),
        }
    }

    pub(super) fn root(&self) -> &Path {
        self.root
    }

    pub(super) fn is_dir(&mut self, relative: &Path) -> bool {
        if let Some(&known) = self.known.get(relative) {
            return known;
        }
        let parent_found = match relative.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => self.is_dir(parent),
            _ => true,
        };
        let found = parent_found && self.root.join(relative).is_dir();
        self.known.insert(relative.to_path_buf(), found);
        found
    }
}
