use crossterm::event::KeyEvent;
use terminput::Event;
use terminput_crossterm::to_crossterm_key;

use crate::types::InputParser;

const MAX_PENDING_BYTES: usize = 1024;

impl InputParser {
    pub fn feed(&mut self, incoming_bytes: &[u8]) -> Vec<KeyEvent> {
        self.pending_bytes.extend_from_slice(incoming_bytes);

        let mut key_events = Vec::new();
        while let Some(event) = self.next_complete_event() {
            if let Event::Key(key_event) = event {
                key_events.push(to_crossterm_key(key_event));
            }
        }
        key_events
    }

    fn next_complete_event(&mut self) -> Option<Event> {
        let mut sequence_length = 1;
        while sequence_length <= self.pending_bytes.len() {
            if sequence_length == 1
                && self.pending_bytes[0] == b'\x1b'
                && self.pending_bytes.len() > 1
            {
                sequence_length += 1;
                continue;
            }
            match Event::parse_from(&self.pending_bytes[..sequence_length]) {
                Ok(Some(event)) => {
                    self.pending_bytes.drain(..sequence_length);
                    return Some(event);
                }
                _ => sequence_length += 1,
            }
        }

        if self.pending_bytes.len() >= MAX_PENDING_BYTES {
            self.pending_bytes.drain(0..1);
        }
        None
    }
}
