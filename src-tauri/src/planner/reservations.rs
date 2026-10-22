use std::{
    collections::{HashMap, HashSet},
    path::Path,
};

use crate::{
    models::{MovePlan, PlanStatus},
    path_safety::normalized_path_string,
};

/// The destination names a scan's plans hold. A file planned again gives its name back
/// first, so it can keep it.
#[derive(Debug, Default)]
pub(crate) struct Reservations {
    paths: HashSet<String>,
    by_file: HashMap<String, String>,
}

impl Reservations {
    pub(super) fn paths(&mut self) -> &mut HashSet<String> {
        &mut self.paths
    }

    pub(crate) fn release(&mut self, file_id: &str) {
        if let Some(path) = self.by_file.remove(file_id) {
            self.paths.remove(&path);
        }
    }

    /// Notes which file holds the name `allocate_destination_path` reserved for `plan`.
    pub(super) fn hold(&mut self, plan: &MovePlan) {
        if matches!(
            plan.plan_status,
            PlanStatus::Ready | PlanStatus::AlreadyNamed
        ) {
            let path = normalized_path_string(Path::new(&plan.destination_path));
            self.by_file.insert(plan.file_id.clone(), path);
        }
    }
}
