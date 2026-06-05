use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::UnboundedSender;

use crate::api::models::DashboardStats;
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

impl<T> Loadable<T> {
    pub fn is_loading(&self) -> bool {
        matches!(self, Loadable::Loading)
    }
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
    tx: UnboundedSender<Event>,
    pub tab: Tab,
    pub should_quit: bool,
    /// Monotonic tick counter driving spinner animation.
    pub ticks: usize,
    /// Transient error shown in the footer until the next keypress.
    pub error: Option<String>,
    pub dashboard: Loadable<DashboardStats>,
}

impl App {
    pub fn new(client: ApiClient, tx: UnboundedSender<Event>) -> Self {
        let mut app = Self {
            client,
            tx,
            tab: Tab::Dashboard,
            should_quit: false,
            ticks: 0,
            error: None,
            dashboard: Loadable::Idle,
        };
        app.refresh();
        app
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Resize => {}
            Event::Tick => self.ticks += 1,
            Event::Data(msg) => self.apply_data(msg),
            Event::Uptime(_) => {}
            Event::Error(message) => {
                self.on_fetch_failed(&message);
                self.error = Some(message);
            }
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
                    self.switch_tab(*tab);
                }
            }
            KeyCode::Char('r') => self.refresh(),
            _ => {}
        }
    }

    fn cycle_tab(&mut self, direction: isize) {
        let count = Tab::ALL.len() as isize;
        let next = (self.tab.index() as isize + direction).rem_euclid(count);
        self.switch_tab(Tab::ALL[next as usize]);
    }

    fn switch_tab(&mut self, tab: Tab) {
        self.tab = tab;
        // Lazily load tab data on first visit; keep cached data otherwise.
        if matches!(self.tab, Tab::Dashboard) && matches!(self.dashboard, Loadable::Idle) {
            self.refresh();
        }
    }

    /// Re-fetch the data behind the current tab.
    fn refresh(&mut self) {
        match self.tab {
            Tab::Dashboard => {
                self.dashboard = Loadable::Loading;
                let client = self.client.clone();
                self.spawn(async move { client.dashboard().await.map(DataMsg::Dashboard) });
            }
            Tab::Entity(_) => {}
        }
    }

    /// Run an API call in the background and feed the result into the event
    /// loop; errors land in `Event::Error`.
    fn spawn<F>(&self, future: F)
    where
        F: Future<Output = color_eyre::eyre::Result<DataMsg>> + Send + 'static,
    {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let event = match future.await {
                Ok(msg) => Event::Data(msg),
                Err(error) => Event::Error(error.to_string()),
            };
            let _ = tx.send(event);
        });
    }

    fn apply_data(&mut self, msg: DataMsg) {
        match msg {
            DataMsg::Dashboard(stats) => self.dashboard = Loadable::Ready(stats),
            _ => {}
        }
    }

    /// Demote any in-flight slot to Failed so spinners don't run forever.
    fn on_fetch_failed(&mut self, message: &str) {
        if self.dashboard.is_loading() {
            self.dashboard = Loadable::Failed(message.to_string());
        }
    }
}
