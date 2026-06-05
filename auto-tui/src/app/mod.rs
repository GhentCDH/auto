pub mod detail;
pub mod list;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::UnboundedSender;

use crate::api::models::DashboardStats;
use crate::api::{ApiClient, EntityKind};
use crate::event::{DataMsg, Event};
use detail::DetailView;
use list::{EntityList, PER_PAGE};

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
    /// One browsing state per entity tab, indexed like `EntityKind::ALL`.
    pub lists: [EntityList; 8],
    /// Drill-down stack of detail views; non-empty means a detail is shown.
    pub detail_stack: Vec<DetailView>,
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
            lists: Default::default(),
            detail_stack: Vec::new(),
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

        // Filter input line captures all typing while focused.
        if let Tab::Entity(kind) = self.tab
            && self.list(kind).filter_input.is_some()
        {
            self.handle_filter_key(kind, key);
            return;
        }

        // An open detail view takes over navigation keys.
        if !self.detail_stack.is_empty() {
            self.handle_detail_key(key);
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
            _ => {
                if let Tab::Entity(kind) = self.tab {
                    self.handle_list_key(kind, key);
                }
            }
        }
    }

    fn handle_list_key(&mut self, kind: EntityKind, key: KeyEvent) {
        let list = self.list_mut(kind);
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => list.move_selection(1),
            KeyCode::Char('k') | KeyCode::Up => list.move_selection(-1),
            KeyCode::Char('g') | KeyCode::Home => list.select_first(),
            KeyCode::Char('G') | KeyCode::End => list.select_last(),
            KeyCode::PageDown => list.move_selection(10),
            KeyCode::PageUp => list.move_selection(-10),
            KeyCode::Char('f') => list.filter_input = Some(list.filter.clone()),
            KeyCode::Enter => {
                if let Some(row) = list.selected_row() {
                    let id = row
                        .get("id")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let seed = row.clone();
                    self.open_detail(kind, id, Some(seed));
                }
            }
            KeyCode::Char('n') | KeyCode::Right => {
                if list.change_page(1) {
                    self.fetch_list(kind);
                }
            }
            KeyCode::Char('p') | KeyCode::Left => {
                if list.change_page(-1) {
                    self.fetch_list(kind);
                }
            }
            _ => {}
        }
    }

    fn handle_filter_key(&mut self, kind: EntityKind, key: KeyEvent) {
        let list = self.list_mut(kind);
        let Some(input) = list.filter_input.as_mut() else {
            return;
        };
        match key.code {
            KeyCode::Esc => list.filter_input = None,
            KeyCode::Enter => {
                list.filter = list.filter_input.take().unwrap_or_default();
                list.page = 1;
                list.select_first();
                self.fetch_list(kind);
            }
            KeyCode::Backspace => {
                input.pop();
            }
            KeyCode::Char(c) => input.push(c),
            _ => {}
        }
    }

    fn handle_detail_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Backspace => {
                self.detail_stack.pop();
            }
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => {
                if let Some(view) = self.detail_stack.last_mut() {
                    view.move_selection(1);
                }
            }
            KeyCode::Char('k') | KeyCode::Up => {
                if let Some(view) = self.detail_stack.last_mut() {
                    view.move_selection(-1);
                }
            }
            KeyCode::Enter => {
                // Drill into the selected related entity.
                if let Some(item) = self.detail_stack.last().and_then(DetailView::selected_item)
                    && let (Some(kind), Some(id)) = (item.kind, item.id)
                {
                    self.open_detail(kind, id, None);
                }
            }
            KeyCode::Char('r') => {
                if let Some(view) = self.detail_stack.last() {
                    let (kind, id) = (view.kind, view.id.clone());
                    self.fetch_detail(kind, id);
                }
            }
            _ => {}
        }
    }

    fn open_detail(&mut self, kind: EntityKind, id: String, seed: Option<serde_json::Value>) {
        if id.is_empty() {
            return;
        }
        self.detail_stack
            .push(DetailView::new(kind, id.clone(), seed));
        self.fetch_detail(kind, id);
    }

    fn fetch_detail(&mut self, kind: EntityKind, id: String) {
        let client = self.client.clone();
        self.spawn(async move {
            let value = client.detail(kind, &id).await?;
            Ok(DataMsg::Detail {
                entity: kind,
                value,
            })
        });
    }

    pub fn list(&self, kind: EntityKind) -> &EntityList {
        &self.lists[Self::list_index(kind)]
    }

    fn list_mut(&mut self, kind: EntityKind) -> &mut EntityList {
        &mut self.lists[Self::list_index(kind)]
    }

    fn list_index(kind: EntityKind) -> usize {
        EntityKind::ALL.iter().position(|k| *k == kind).unwrap_or(0)
    }

    fn fetch_list(&mut self, kind: EntityKind) {
        let list = self.list_mut(kind);
        list.data = Loadable::Loading;
        let (page, filter) = (list.page.max(1), list.filter.clone());
        let client = self.client.clone();
        self.spawn(async move {
            let resp = client.list(kind, page, PER_PAGE, &filter).await?;
            Ok(DataMsg::List { entity: kind, resp })
        });
    }

    fn cycle_tab(&mut self, direction: isize) {
        let count = Tab::ALL.len() as isize;
        let next = (self.tab.index() as isize + direction).rem_euclid(count);
        self.switch_tab(Tab::ALL[next as usize]);
    }

    fn switch_tab(&mut self, tab: Tab) {
        self.tab = tab;
        // Lazily load tab data on first visit; keep cached data otherwise.
        let idle = match self.tab {
            Tab::Dashboard => matches!(self.dashboard, Loadable::Idle),
            Tab::Entity(kind) => matches!(self.list(kind).data, Loadable::Idle),
        };
        if idle {
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
            Tab::Entity(kind) => self.fetch_list(kind),
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
            DataMsg::List { entity, resp } => {
                let list = self.list_mut(entity);
                list.selected = list.selected.min(resp.data.len().saturating_sub(1));
                list.data = Loadable::Ready(resp);
            }
            DataMsg::Detail { entity, value } => {
                let id = value
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or_default();
                // Route to the matching stack entry (usually the top).
                if let Some(view) = self
                    .detail_stack
                    .iter_mut()
                    .rev()
                    .find(|view| view.kind == entity && view.id == id)
                {
                    view.data = Loadable::Ready(value);
                }
            }
            _ => {}
        }
    }

    /// Demote any in-flight slot to Failed so spinners don't run forever.
    fn on_fetch_failed(&mut self, message: &str) {
        if self.dashboard.is_loading() {
            self.dashboard = Loadable::Failed(message.to_string());
        }
        for list in &mut self.lists {
            if list.data.is_loading() {
                list.data = Loadable::Failed(message.to_string());
            }
        }
        if let Some(view) = self.detail_stack.last_mut()
            && view.data.is_loading()
        {
            view.data = Loadable::Failed(message.to_string());
        }
    }
}
