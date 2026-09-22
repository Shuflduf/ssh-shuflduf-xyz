use std::{collections::HashMap, sync::Arc};

use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};
use tokio::sync::{Mutex, broadcast, mpsc::UnboundedSender};

pub type SshTerminal = Terminal<CrosstermBackend<TerminalHandle>>;

#[derive(Default)]
pub struct AppState {
    pub counter: i32,
    pub should_exit: bool,
}

pub enum ClientEvent {
    Input(Vec<u8>),
    Resize(Rect),
}

#[derive(Clone)]
pub enum ServerEvent {
    Increment,
    Decrement,
}

#[derive(Clone)]
pub struct ServerState {
    pub current_value: Arc<Mutex<i32>>,
    pub broadcast_sender: broadcast::Sender<ServerEvent>,
}

impl Default for ServerState {
    fn default() -> Self {
        let (broadcast_sender, _receiver) = broadcast::channel(64);
        Self {
            current_value: Arc::new(Mutex::new(0)),
            broadcast_sender,
        }
    }
}

pub struct TerminalHandle {
    pub terminal_bytes_sender: UnboundedSender<Vec<u8>>,
    pub pending_bytes: Vec<u8>,
}

#[derive(Default)]
pub struct InputParser {
    pub pending_bytes: Vec<u8>,
}

#[derive(Clone)]
pub struct AppServer {
    pub clients: Arc<Mutex<HashMap<usize, UnboundedSender<ClientEvent>>>>,
    pub server_state: ServerState,
    pub client_id: usize,
}

impl Default for AppServer {
    fn default() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            server_state: ServerState::default(),
            client_id: 0,
        }
    }
}
