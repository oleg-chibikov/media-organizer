use std::sync::Arc;

use super::ScanSession;
use crate::{
    events::EventSink,
    models::{MetadataReadRequest, MetadataResult},
    scan_metadata::BatchReader,
};

/// Keeps every date it reads in the scan's session and plans those files, before the
/// webview hears of the dates.
pub(crate) struct SessionReader<R, S> {
    pub inner: R,
    pub session: Arc<ScanSession>,
    pub sink: S,
}

impl<R: BatchReader, S: EventSink + Send + Sync + 'static> BatchReader for SessionReader<R, S> {
    fn read(&self, requests: Vec<MetadataReadRequest>) -> Vec<MetadataResult> {
        let results = self.inner.read(requests);
        self.session.add_metadata(&results);
        self.session.plan_read(&self.sink, &results);
        results
    }
}
