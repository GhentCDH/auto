use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::api::{ApiClient, EntityKind};
use crate::event::{DataMsg, Event};

/// One slot of remotely-fetched screen data.
#[derive(Debug, Default)]
pub enum Loadable<T> {
    #[default]
    Idle,
    Loading,
    Ready(T),
    Failed(String),
}

/// Top-level tabs: dashboard plus one tab per entity kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Entity(EntityKind),
}

impl Tab {
    pub const ALL: [Tab; 9] = [
        Tab::Dashboard,
        Tab::Entity(EntityKind::Applications),
        Tab::Entity(EntityKind::Services),
        Tab::Entity(EntityKind::Infra),
        Tab::Entity(EntityKind::Domains),
        Tab::Entity(EntityKind::People),
        Tab::Entity(EntityKind::Shares),
        Tab::Entity(EntityKind::Stacks),
        Tab::Entity(EntityKind::Healthchecks),
    ];

    pub fn label(self) -> &'static str {
        match self {
            Tab::Dashboard => "Dashboard",
            Tab::Entity(kind) => kind.label(),
        }
    }

    fn index(self) -> usize {
        Tab::ALL.iter().position(|t| *t == self).unwrap_or(0)
    }
}

pub struct App {
    pub client: ApiClient,
    pub tab: Tab,
    pub should_quit: bool,
    /// Monotonic tick counter driving spinner animation.
    pub ticks: usize,
    /// Transient error shown in the footer until the next keypress.
    pub error: Option<String>,
}

impl App {
    pub fn new(client: ApiClient) -> Self {
        Self {
            client,
            tab: Tab::Dashboard,
            should_quit: false,
            ticks: 0,
            error: None,
        }
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Resize => {}
            Event::Tick => self.ticks += 1,
            Event::Data(msg) => self.apply_data(msg),
            Event::Uptime(_) => {}
            Event::Error(message) => self.error = Some(message),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Any keypress clears a stale error toast.
        self.error = None;

        if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
            self.should_quit = true;
            return;
        }
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => self.cycle_tab(1),
            KeyCode::BackTab => self.cycle_tab(-1),
            KeyCode::Char(c @ '1'..='9') => {
                let index = c as usize - '1' as usize;
                if let Some(tab) = Tab::ALL.get(index) {
                    self.tab = *tab;
                }
            }
            _ => {}
        }
    }

    fn cycle_tab(&mut self, direction: isize) {
        let count = Tab::ALL.len() as isize;
        let next = (self.tab.index() as isize + direction).rem_euclid(count);
        self.tab = Tab::ALL[next as usize];
    }

    fn apply_data(&mut self, _msg: DataMsg) {}
}
