//! Pre-UI login flow.
//!
//! Runs before the main UI when the client is not authenticated. The primary
//! action is a browser-loopback login (works with password and OIDC/SSO);
//! username/password is offered as a secondary option. On success the shared
//! [`ApiClient`] carries the session and the main UI starts.

use color_eyre::eyre::Result;
use crossterm::event::{Event as CtEvent, KeyCode, KeyEvent, KeyModifiers};
use futures::StreamExt;
use tokio::sync::mpsc;

use crate::api::ApiClient;
use crate::ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Browser,
    Password,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Username,
    Password,
}

pub struct LoginView {
    pub url: String,
    pub reason: Option<String>,
    pub mode: Mode,
    pub username: String,
    pub password: String,
    pub field: Field,
    pub status: String,
    pub error: Option<String>,
    pub browser_pending: bool,
    pub quit: bool,
    pub authenticated: bool,
}

impl LoginView {
    fn new(
        url: String,
        reason: Option<String>,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Self {
        Self {
            url,
            reason,
            mode: Mode::Browser,
            username: username.unwrap_or_default().to_string(),
            password: password.unwrap_or_default().to_string(),
            field: if username.is_some() {
                Field::Password
            } else {
                Field::Username
            },
            status: String::new(),
            error: None,
            browser_pending: false,
            quit: false,
            authenticated: false,
        }
    }
}

/// Run the login screen until the user authenticates or quits. Returns `true`
/// when authentication succeeded.
pub async fn run(
    terminal: &mut ratatui::DefaultTerminal,
    client: &ApiClient,
    reason: Option<String>,
) -> Result<bool> {
    let mut view = LoginView::new(
        client.server_url().to_string(),
        reason,
        client.username(),
        client.password(),
    );
    let (tx, mut rx) = mpsc::unbounded_channel::<Result<String, String>>();
    let mut events = crossterm::event::EventStream::new();

    loop {
        terminal.draw(|frame| ui::login::draw(frame, &view))?;

        tokio::select! {
            event = events.next() => match event {
                Some(Ok(CtEvent::Key(key))) => handle_key(key, &mut view, client, &tx).await,
                Some(Ok(_)) => {}
                Some(Err(_)) | None => view.quit = true,
            },
            Some(result) = rx.recv() => {
                view.browser_pending = false;
                match result {
                    Ok(_) => view.authenticated = true,
                    Err(error) => {
                        view.status.clear();
                        view.error = Some(error);
                    }
                }
            }
        }

        if view.authenticated {
            return Ok(true);
        }
        if view.quit {
            return Ok(false);
        }
    }
}

async fn handle_key(
    key: KeyEvent,
    view: &mut LoginView,
    client: &ApiClient,
    tx: &mpsc::UnboundedSender<Result<String, String>>,
) {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        view.quit = true;
        return;
    }
    if key.code == KeyCode::Char('q') && view.mode == Mode::Browser && !view.browser_pending {
        view.quit = true;
        return;
    }

    match view.mode {
        Mode::Browser => match key.code {
            KeyCode::Enter => {
                if !view.browser_pending {
                    start_browser(view, client, tx).await;
                }
            }
            KeyCode::Down | KeyCode::Tab | KeyCode::Char('j') => view.mode = Mode::Password,
            _ => {}
        },
        Mode::Password => match key.code {
            KeyCode::Esc => view.mode = Mode::Browser,
            KeyCode::Tab | KeyCode::Down | KeyCode::Up | KeyCode::BackTab => toggle_field(view),
            KeyCode::Enter => submit_password(view, client).await,
            KeyCode::Backspace => {
                active_field(view).pop();
            }
            KeyCode::Char(c) => active_field(view).push(c),
            _ => {}
        },
    }
}

fn active_field(view: &mut LoginView) -> &mut String {
    match view.field {
        Field::Username => &mut view.username,
        Field::Password => &mut view.password,
    }
}

fn toggle_field(view: &mut LoginView) {
    view.field = match view.field {
        Field::Username => Field::Password,
        Field::Password => Field::Username,
    };
}

async fn start_browser(
    view: &mut LoginView,
    client: &ApiClient,
    tx: &mpsc::UnboundedSender<Result<String, String>>,
) {
    view.error = None;
    view.status = "Starting browser login…".to_string();
    match client.start_loopback().await {
        Ok(session) => {
            let url = session.authorize_url.clone();
            match open::that(&url) {
                Ok(()) => view.status = "Waiting for the browser to finish sign-in…".to_string(),
                Err(_) => view.status = format!("Open this URL to sign in: {url}"),
            }
            view.browser_pending = true;
            let client = client.clone();
            let tx = tx.clone();
            tokio::spawn(async move {
                let result = session.finish(&client).await.map_err(|e| e.to_string());
                let _ = tx.send(result);
            });
        }
        Err(error) => {
            view.status.clear();
            view.error = Some(error.to_string());
        }
    }
}

async fn submit_password(view: &mut LoginView, client: &ApiClient) {
    if view.username.is_empty() || view.password.is_empty() {
        view.error = Some("enter username and password".to_string());
        return;
    }
    view.error = None;
    view.status = "Signing in…".to_string();
    match client.login_password(&view.username, &view.password).await {
        Ok(()) => view.authenticated = true,
        Err(error) => {
            view.status.clear();
            view.error = Some(error.to_string());
        }
    }
}
