use std::{collections::HashMap, sync::Arc};

use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};
use tokio::sync::{Mutex, mpsc::UnboundedSender};

pub type SshTerminal = Terminal<CrosstermBackend<TerminalHandle>>;

#[derive(Default)]
pub struct AppState {
    pub counter: usize,
}

pub enum ClientEvent {
    Input(Vec<u8>),
    Resize(Rect),
}

pub struct Client {
    pub terminal: SshTerminal,
    pub state: AppState,
    pub events: UnboundedSender<ClientEvent>,
}

pub struct TerminalHandle {
    pub sender: UnboundedSender<Vec<u8>>,
    // The sink collects the data which is finally sent to sender.
    pub sink: Vec<u8>,
}

#[derive(Clone, Default)]
pub struct AppServer {
    pub clients: Arc<Mutex<HashMap<usize, (SshTerminal, AppState)>>>,
    pub id: usize,
}
