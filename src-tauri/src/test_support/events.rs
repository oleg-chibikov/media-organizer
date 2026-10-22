use std::sync::Mutex;

use serde::Serialize;
use serde_json::Value;

use crate::events::EventSink;

/// Keeps every emitted event in order so tests can assert on the stream.
#[derive(Default)]
pub(crate) struct RecordingSink {
    events: Mutex<Vec<(&'static str, Value)>>,
}

impl EventSink for RecordingSink {
    fn emit<P: Serialize + Clone>(&self, event: &'static str, payload: P) {
        let payload = serde_json::to_value(payload).expect("event payload should serialize");
        self.events
            .lock()
            .expect("recording sink lock")
            .push((event, payload));
    }
}

/// Lets a test keep the sink while background threads emit into it.
impl EventSink for std::sync::Arc<RecordingSink> {
    fn emit<P: Serialize + Clone>(&self, event: &'static str, payload: P) {
        self.as_ref().emit(event, payload);
    }
}

impl RecordingSink {
    pub(crate) fn names(&self) -> Vec<&'static str> {
        self.events().into_iter().map(|(name, _)| name).collect()
    }

    pub(crate) fn payloads(&self, event: &str) -> Vec<Value> {
        self.events()
            .into_iter()
            .filter(|(name, _)| *name == event)
            .map(|(_, payload)| payload)
            .collect()
    }

    fn events(&self) -> Vec<(&'static str, Value)> {
        self.events.lock().expect("recording sink lock").clone()
    }
}
