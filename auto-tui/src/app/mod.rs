pub mod detail;
pub mod list;
pub mod search;
pub mod uptime;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc::UnboundedSender;

use crate::api::models::{DashboardStats, DnsLookup, HealthcheckExecuteResult};
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
    /// Live Kuma heartbeat state from the SSE stream.
    pub uptime: uptime::UptimeState,
    /// Transient success message with its expiry tick.
    pub toast: Option<(String, usize)>,
    /// Healthcheck execute result popup (Loading while running).
    pub exec_result: Option<Loadable<HealthcheckExecuteResult>>,
    /// Global search overlay.
    pub search: search::SearchState,
    /// DNS lookup overlay for a domain: (fqdn label, lookup result).
    pub dns: Option<(String, Loadable<DnsLookup>)>,
    /// Help overlay toggle (`?`).
    pub show_help: bool,
    /// Dashboard health section: collapsed toggle (`h`) and scroll offset.
    pub health_collapsed: bool,
    pub health_scroll: u16,
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
            uptime: uptime::UptimeState::default(),
            toast: None,
            exec_result: None,
            search: search::SearchState::default(),
            dns: None,
            show_help: false,
            health_collapsed: false,
            health_scroll: 0,
        };
        app.refresh();
        app
    }

    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::Key(key) => self.handle_key(key),
            Event::Resize => {}
            Event::Tick => {
                self.ticks += 1;
                if let Some((_, expires)) = &self.toast
                    && self.ticks >= *expires
                {
                    self.toast = None;
                }
            }
            Event::Data(msg) => self.apply_data(msg),
            Event::Uptime(event) => self.uptime.apply(event),
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

        // Help overlay: any key closes it.
        if self.show_help {
            self.show_help = false;
            return;
        }

        // Overlays take precedence: search, then DNS view.
        if self.search.open {
            self.handle_search_key(key);
            return;
        }
        if self.dns.is_some() {
            match key.code {
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => self.dns = None,
                _ => {}
            }
            return;
        }

        // An open execute-result popup swallows the next keypress.
        if self.exec_result.is_some() {
            if !matches!(self.exec_result, Some(Loadable::Loading)) {
                self.exec_result = None;
            }
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
            KeyCode::Char('/') => self.search.open = true,
            KeyCode::Char('?') => self.show_help = true,
            _ => match self.tab {
                Tab::Entity(kind) => self.handle_list_key(kind, key),
                Tab::Dashboard => self.handle_dashboard_key(key),
            },
        }
    }

    fn handle_dashboard_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('h') => {
                self.health_collapsed = !self.health_collapsed;
                self.health_scroll = 0;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                // Upper bound enforced at draw time against the line count.
                let max = self.list(EntityKind::Healthchecks).rows().len() as u16;
                self.health_scroll = (self.health_scroll + 1).min(max);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.health_scroll = self.health_scroll.saturating_sub(1);
            }
            _ => {}
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => self.search.close(),
            KeyCode::Down => self.search.move_selection(1),
            KeyCode::Up => self.search.move_selection(-1),
            KeyCode::Enter => {
                if let Some((kind, result)) = self.search.flat().get(self.search.selected) {
                    let (kind, id) = (*kind, result.id.clone());
                    self.search.close();
                    self.open_detail(kind, id, None);
                }
            }
            KeyCode::Backspace => {
                self.search.input.pop();
                self.fire_search();
            }
            KeyCode::Char(c) => {
                self.search.input.push(c);
                self.fire_search();
            }
            _ => {}
        }
    }

    /// Search-as-you-type; results route back via `DataMsg::Search`.
    fn fire_search(&mut self) {
        if self.search.input.trim().is_empty() {
            self.search.results = Loadable::Idle;
            return;
        }
        self.search.results = Loadable::Loading;
        self.search.selected = 0;
        let query = self.search.input.clone();
        let client = self.client.clone();
        self.spawn(async move { client.search(&query).await.map(DataMsg::Search) });
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
            _ => self.handle_action_key(kind, key, self.list(kind).selected_id()),
        }
    }

    /// Safe actions: healthcheck execute / kuma sync, infra sync.
    fn handle_action_key(&mut self, kind: EntityKind, key: KeyEvent, id: Option<String>) {
        match (kind, key.code) {
            (EntityKind::Healthchecks, KeyCode::Char('x')) => {
                let Some(id) = id else { return };
                self.exec_result = Some(Loadable::Loading);
                let client = self.client.clone();
                self.spawn(async move {
                    client
                        .execute_healthcheck(&id)
                        .await
                        .map(DataMsg::ExecResult)
                });
            }
            (EntityKind::Healthchecks, KeyCode::Char('s')) => {
                let Some(id) = id else { return };
                let client = self.client.clone();
                self.spawn(async move {
                    client.kuma_sync_one(&id).await?;
                    Ok(DataMsg::ActionDone {
                        label: "healthcheck synced to Kuma".into(),
                    })
                });
            }
            (EntityKind::Healthchecks, KeyCode::Char('S')) => {
                let client = self.client.clone();
                self.spawn(async move {
                    client.kuma_sync_all().await?;
                    Ok(DataMsg::ActionDone {
                        label: "all healthchecks synced to Kuma".into(),
                    })
                });
            }
            (EntityKind::Infra, KeyCode::Char('s')) => {
                let Some(id) = id else { return };
                let client = self.client.clone();
                self.spawn(async move {
                    client.infra_sync_one(&id).await?;
                    Ok(DataMsg::ActionDone {
                        label: "infra IPs synced".into(),
                    })
                });
            }
            (EntityKind::Domains, KeyCode::Char('d')) => {
                let Some(id) = id else { return };
                // Label from the list row or detail view, falling back to id.
                let label = self
                    .list(EntityKind::Domains)
                    .selected_row()
                    .filter(|row| row.get("id").and_then(serde_json::Value::as_str) == Some(&id))
                    .or_else(|| self.detail_stack.last().and_then(DetailView::value))
                    .map(list::row_label)
                    .unwrap_or(&id)
                    .to_string();
                self.dns = Some((label, Loadable::Loading));
                let client = self.client.clone();
                self.spawn(async move { client.dns(&id).await.map(DataMsg::Dns) });
            }
            (EntityKind::Infra, KeyCode::Char('S')) => {
                let client = self.client.clone();
                self.spawn(async move {
                    client.infra_sync_all().await?;
                    Ok(DataMsg::ActionDone {
                        label: "all infra IPs synced".into(),
                    })
                });
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
            _ => {
                // Context actions on the detailed entity itself.
                if let Some(view) = self.detail_stack.last() {
                    let (kind, id) = (view.kind, view.id.clone());
                    self.handle_action_key(kind, key, Some(id));
                }
            }
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
                // The health section reuses the healthchecks tab data.
                if matches!(
                    self.list(EntityKind::Healthchecks).data,
                    Loadable::Idle | Loadable::Failed(_)
                ) {
                    self.fetch_list(EntityKind::Healthchecks);
                }
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
            DataMsg::Search(results) => {
                if self.search.open {
                    self.search.results = Loadable::Ready(results);
                }
            }
            DataMsg::Dns(lookup) => {
                if let Some((_, slot)) = self.dns.as_mut() {
                    *slot = Loadable::Ready(lookup);
                }
            }
            DataMsg::ExecResult(result) => {
                if self.exec_result.is_some() {
                    self.exec_result = Some(Loadable::Ready(result));
                }
            }
            DataMsg::ActionDone { label } => {
                // Toast for ~3 seconds (ticks are 250 ms).
                self.toast = Some((label, self.ticks + 12));
            }
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
        if matches!(self.exec_result, Some(Loadable::Loading)) {
            self.exec_result = Some(Loadable::Failed(message.to_string()));
        }
        if self.search.results.is_loading() {
            self.search.results = Loadable::Failed(message.to_string());
        }
        if let Some((_, slot)) = self.dns.as_mut()
            && slot.is_loading()
        {
            *slot = Loadable::Failed(message.to_string());
        }
    }
}
