use std::{
    io::{self, BufRead, BufReader, Read},
    path::Path,
};

use crate::models::HistoryRecord;

/// A non-empty run log line with its line terminator removed.
pub(super) struct LogLine {
    pub number: usize,
    bytes: Vec<u8>,
    terminated: bool,
}

pub(super) enum LineDecodeError {
    /// The final line was cut short by a crash mid-write.
    Truncated,
    Malformed(String),
}

pub(super) struct RunLogLines<R> {
    reader: BufReader<R>,
    line_number: usize,
}

impl<R: Read> RunLogLines<R> {
    pub(super) fn new(reader: R) -> Self {
        Self {
            reader: BufReader::new(reader),
            line_number: 0,
        }
    }

    pub(super) fn next_line(&mut self) -> io::Result<Option<LogLine>> {
        loop {
            let mut bytes = Vec::new();
            if self.reader.read_until(b'\n', &mut bytes)? == 0 {
                return Ok(None);
            }

            self.line_number += 1;
            let terminated = bytes.last() == Some(&b'\n');
            if terminated {
                bytes.pop();
            }
            if bytes.last() == Some(&b'\r') {
                bytes.pop();
            }
            if !bytes.is_empty() {
                return Ok(Some(LogLine {
                    number: self.line_number,
                    bytes,
                    terminated,
                }));
            }
        }
    }
}

impl LogLine {
    pub(super) fn decode(&self, log_path: &Path) -> Result<HistoryRecord, LineDecodeError> {
        let text = std::str::from_utf8(&self.bytes).map_err(|error| {
            self.failure(format!(
                "Malformed UTF-8 at line {} in '{}': {error}",
                self.number,
                log_path.display()
            ))
        })?;

        serde_json::from_str(text).map_err(|error| {
            self.failure(format!(
                "Malformed history record at line {} in '{}': {error}",
                self.number,
                log_path.display()
            ))
        })
    }

    fn failure(&self, message: String) -> LineDecodeError {
        if self.terminated {
            LineDecodeError::Malformed(message)
        } else {
            LineDecodeError::Truncated
        }
    }
}
