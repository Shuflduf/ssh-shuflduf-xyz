use russh::{ChannelId, server::Handle};
use tokio::sync::mpsc::unbounded_channel;

use crate::types::TerminalHandle;

impl TerminalHandle {
    pub async fn start(connection_handle: Handle, channel_id: ChannelId) -> Self {
        let (terminal_bytes_sender, mut bytes_receiver) = unbounded_channel::<Vec<u8>>();
        tokio::spawn(async move {
            while let Some(bytes) = bytes_receiver.recv().await {
                if let Err(error) = connection_handle.data(channel_id, bytes).await {
                    eprintln!("Failed to send terminal output to client: {error:?}");
                }
            }
        });
        Self {
            terminal_bytes_sender,
            pending_bytes: Vec::new(),
        }
    }
}

impl std::io::Write for TerminalHandle {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.pending_bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let bytes = std::mem::take(&mut self.pending_bytes);
        self.terminal_bytes_sender
            .send(bytes)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::BrokenPipe, "client disconnected"))
    }
}
