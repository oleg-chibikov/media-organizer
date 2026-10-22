use std::sync::Arc;

use super::ScanSession;
use crate::{
    models::{MetadataReadRequest, MetadataResult},
    scan_metadata::BatchReader,
};

/// Keeps every date it reads in the scan's session, before the webview hears of it.
pub(crate) struct SessionReader<R> {
    pub inner: R,
    pub session: Arc<ScanSession>,
}

impl<R: BatchReader> BatchReader for SessionReader<R> {
    fn read(&self, requests: Vec<MetadataReadRequest>) -> Vec<MetadataResult> {
        let results = self.inner.read(requests);
        self.session.add_metadata(&results);
        results
    }
}
