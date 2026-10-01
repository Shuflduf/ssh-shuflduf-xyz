use std::{collections::HashMap, sync::Arc};

use crossterm::event::KeyEvent;
use ratatui::{Terminal, backend::CrosstermBackend, layout::Rect, style::Color};
use strum::{Display, EnumCount, EnumIter, EnumProperty, VariantArray};
use tokio::sync::{Mutex, broadcast, mpsc::UnboundedSender};

use crate::{
    app::{chat::Chat, counter::Counter},
    games::{
        snake::{Snake, SnakeScheme},
        tetris::{Tetris, TetrisScheme},
        wordle::{Wordle, WordleScheme},
    },
};

pub type SshTerminal = Terminal<CrosstermBackend<TerminalHandle>>;

// client stuff

#[derive(
    Default, Debug, EnumIter, EnumCount, Display, VariantArray, PartialEq, Eq, Clone, Copy,
)]
pub enum Content {
    #[default]
    Counter,
    Games,
    Chat,
}

#[derive(Default, PartialEq, Eq)]
pub enum Focus {
    Sidebar,
    #[default]
    Pane,
}

#[derive(Default)]
pub struct Sidebar {
    pub item: Content,
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

#[derive(Default)]
pub struct Games {
    pub wordle: Option<Wordle>,
    pub snake: Option<Snake>,
    pub tetris: Option<Tetris>,

    pub focused_game: Game,
    pub active_game: Option<Game>,
}

#[derive(Default)]
pub struct ClientState {
    pub sidebar: Sidebar,
    pub counter: Counter,
    pub games: Games,
    pub chat: Chat,

    pub current_pane: Content,
    pub focus: Focus,
    pub exiting: bool,
}

pub struct ColourScheme {
    pub base: Color,
    pub surface: Color,
    pub surface_secondary: Color,
    pub text: Color,
    pub text_secondary: Color,
    pub keys: Color,
    pub accent: Color,

    pub wordle: WordleScheme,
    pub snake: SnakeScheme,
    pub tetris: TetrisScheme,
}

// server stuff

#[derive(Clone)]
pub enum ServerMessage {
    CounterIncrement,
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
