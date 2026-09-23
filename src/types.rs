use std::{collections::HashMap, sync::Arc};

use crossterm::event::KeyEvent;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect};
use strum::{Display, EnumCount, EnumIter, EnumProperty, VariantArray};
use tokio::{
    sync::{Mutex, broadcast, mpsc::UnboundedSender},
    time::Instant,
};

pub type SshTerminal = Terminal<CrosstermBackend<TerminalHandle>>;

// client stuff

#[derive(Default)]
pub enum MainPane {
    #[default]
    Counter,
    Games,
}

#[derive(Default, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    #[default]
    Pane,
}

#[derive(
    Default, Debug, EnumIter, EnumCount, Display, VariantArray, PartialEq, Eq, Clone, Copy,
)]
pub enum SidebarItem {
    #[default]
    Counter,
    Games,
}

#[derive(Default)]
pub struct Sidebar {
    pub item: SidebarItem,
}

#[derive(Default)]
pub struct Counter {
    pub count: u32,
    pub last_increment_time: Option<Instant>,
}

#[derive(
    Debug,
    EnumIter,
    EnumCount,
    Display,
    EnumProperty,
    Default,
    PartialEq,
    Eq,
    VariantArray,
    Clone,
    Copy,
)]
pub enum Game {
    #[default]
    #[strum(props(Description = "Try to guess a word"))]
    Wordle,
    #[strum(props(Description = "Consume fruit to grow longer"))]
    Snake,
    #[strum(props(Description = "Place falling blocks in a stack"))]
    Tetris,
}

pub struct Wordle {
    pub correct_word: String,
    pub guesses: Vec<String>,
    pub current_guess: String,
}

#[derive(Default)]
pub struct Games {
    pub wordle: Option<Wordle>,

    pub focused_game: Game,
    pub active_game: Option<Game>,
}

#[derive(Default)]
pub struct ClientState {
    pub sidebar: Sidebar,
    pub counter: Counter,
    pub games: Games,

    pub current_pane: MainPane,
    pub focus: Focus,
    pub exiting: bool,
}

// server stuff

#[derive(Clone)]
pub enum ServerMessage {
    Increment,
}

#[derive(Clone)]
pub struct ServerState {
    pub current_value: Arc<Mutex<u32>>,
    pub broadcast_sender: broadcast::Sender<ServerMessage>,
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

pub enum ClientEvent {
    Input(Vec<u8>),
    Resize(Rect),
}

#[derive(Clone)]
pub enum ClientMessage {
    KeyPressed(KeyEvent),
    TerminalResized(Rect),
}

pub struct TerminalHandle {
    pub terminal_bytes_sender: UnboundedSender<Vec<u8>>,
    pub pending_bytes: Vec<u8>,
}

#[derive(Default)]
pub struct InputParser {
    pub pending_bytes: Vec<u8>,
}

#[derive(Clone, Default)]
pub struct AppServer {
    pub clients: Arc<Mutex<HashMap<usize, UnboundedSender<ClientEvent>>>>,
    pub server_state: ServerState,
    pub client_id: usize,
}
