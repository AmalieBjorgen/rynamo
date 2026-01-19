//! Rynamo - A TUI explorer for Dataverse and Dynamics 365
//!
//! This tool allows you to explore Dataverse metadata, including:
//! - Entity definitions (tables)
//! - Attributes (columns)
//! - Relationships
//! - Solutions

mod api;
mod auth;
mod models;
mod ui;

use anyhow::{Context, Result};
use clap::Parser;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::DataverseClient;
use crate::auth::AzureAuthenticator;
use crate::ui::{App, InputMode, KeyBindings, View};

/// Rynamo - Dataverse TUI Explorer
#[derive(Parser, Debug)]
#[command(name = "rynamo")]
#[command(about = "A terminal UI for exploring Dataverse and Dynamics 365 metadata")]
#[command(version)]
struct Args {
    /// Dataverse environment URL (e.g., https://yourorg.crm.dynamics.com)
    #[arg(short, long, env = "DATAVERSE_URL")]
    env: String,

    /// Use vim-style keybindings (j/k navigation)
    #[arg(long, default_value = "false")]
    vim: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (for debugging, set RUST_LOG=debug)
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer().with_writer(io::stderr))
        .init();

    let args = Args::parse();

    // Set up authentication
    let authenticator = Arc::new(
        AzureAuthenticator::new(&args.env)
            .await
            .context("Failed to create Azure authenticator")?,
    );

    // Test connection before starting TUI
    eprintln!("Connecting to {}...", args.env);
    authenticator
        .test_connection()
        .await
        .context("Failed to authenticate. Make sure you're logged in with 'az login'")?;
    eprintln!("Connected successfully!");

    // Create API client
    let client = Arc::new(DataverseClient::new(authenticator));

    // Set up key bindings
    let key_bindings = if args.vim {
        KeyBindings::Vim
    } else {
        KeyBindings::Arrows
    };

    // Set up terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run
    let mut app = App::new(client, key_bindings);
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(e) = result {
        eprintln!("Error: {:?}", e);
    }

    Ok(())
}

/// Main event loop
async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> Result<()> {
    // Load initial data
    app.load_entities().await;

    loop {
        // Render
        terminal.draw(|f| ui::components::render(f, app))?;

        // Handle events with timeout
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                match app.input_mode {
                    InputMode::Normal => handle_normal_mode(app, key.code).await,
                    InputMode::Search => handle_search_mode(app, key.code),
                }

                if app.should_quit {
                    break;
                }
            }
        }
    }

    Ok(())
}

/// Handle input in normal mode
async fn handle_normal_mode(app: &mut App, key: KeyCode) {
    use crate::ui::EntityTab;
    
    // Global shortcuts
    match key {
        KeyCode::Char('q') => {
            // Only quit from main views, go back from detail views
            match app.view {
                View::EntityDetail | View::SolutionDetail | View::UserDetail => app.go_back(),
                _ => app.should_quit = true,
            }
            return;
        }
        KeyCode::Char('/') => {
            app.input_mode = InputMode::Search;
            app.search_query.clear();
            return;
        }
        KeyCode::Esc => {
            app.go_back();
            return;
        }
        KeyCode::Char('1') => {
            if app.view != View::Entities && app.view != View::EntityDetail {
                app.view = View::Entities;
            }
            return;
        }
        KeyCode::Char('2') => {
            if app.view != View::Solutions && app.view != View::SolutionDetail {
                app.view = View::Solutions;
                if app.solutions.is_empty() {
                    app.load_solutions().await;
                }
            }
            return;
        }
        KeyCode::Char('3') => {
            if app.view != View::Users && app.view != View::UserDetail {
                app.view = View::Users;
                if app.users.is_empty() {
                    app.load_users().await;
                }
            }
            return;
        }
        _ => {}
    }

    // Navigation
    if app.key_bindings.is_up(key) {
        app.navigate_up();
        return;
    }
    if app.key_bindings.is_down(key) {
        app.navigate_down();
        return;
    }
    if app.key_bindings.is_left(key) {
        app.prev_tab();
        return;
    }
    if app.key_bindings.is_right(key) {
        app.next_tab();
        return;
    }

    // Query tab specific keys
    if app.view == View::EntityDetail && app.entity_tab == EntityTab::Query {
        match key {
            KeyCode::Enter => {
                match app.query_mode {
                    crate::ui::QueryMode::Columns => {
                        // Start adding filter for selected column
                        app.query_filter_attr = Some(app.query_column_index);
                        app.query_mode = crate::ui::QueryMode::Filter;
                    }
                    crate::ui::QueryMode::Filter => {
                        // Add current filter
                        app.add_filter();
                    }
                    crate::ui::QueryMode::Results => {
                        app.enter_record_detail();
                    }
                    _ => {
                        // Execute query in other modes
                        app.execute_guided_query().await;
                    }
                }
            }
            KeyCode::Char('d') => {
                if app.query_mode == crate::ui::QueryMode::Filter {
                    app.remove_filter();
                }
            }
            KeyCode::Char('o') => {
                if app.query_mode == crate::ui::QueryMode::Filter {
                    app.query_filter_op = app.query_filter_op.next();
                }
            }
            KeyCode::Char('O') => {
                if app.query_mode == crate::ui::QueryMode::Filter {
                    app.query_filter_op = app.query_filter_op.prev();
                }
            }
            KeyCode::Backspace => {
                if app.query_mode == crate::ui::QueryMode::Filter {
                    app.query_filter_value.pop();
                }
            }
            KeyCode::Char(c) => {
                if app.query_mode == crate::ui::QueryMode::Filter && app.query_filter_attr.is_some() {
                    app.query_filter_value.push(c);
                } else {
                    // Fallback to original handlers for things like ' ' (space)
                    match c {
                        ' ' => app.toggle_query_column(),
                        'a' => app.select_all_columns(),
                        'c' => app.clear_query(),
                        'n' => {
                            if app.query_mode == crate::ui::QueryMode::Results {
                                app.load_next_page().await;
                            }
                        }
                        _ => {}
                    }
                }
            }
            KeyCode::F(5) => {
                // Execute query
                app.execute_guided_query().await;
            }
            KeyCode::Tab => {
                // Switch query mode
                app.query_mode = match app.query_mode {
                    crate::ui::QueryMode::Columns => crate::ui::QueryMode::Filter,
                    crate::ui::QueryMode::Filter => crate::ui::QueryMode::Options,
                    crate::ui::QueryMode::Options => crate::ui::QueryMode::Results,
                    crate::ui::QueryMode::OrderBy => crate::ui::QueryMode::Results,
                    crate::ui::QueryMode::Results => crate::ui::QueryMode::Columns,
                };
            }
            KeyCode::Esc => {
                if app.query_mode == crate::ui::QueryMode::Filter && app.query_filter_attr.is_some() {
                    app.query_filter_attr = None;
                } else {
                    app.go_back();
                }
            }
            _ => {}
        }
        return;
    }

    // Enter to select
    if key == KeyCode::Enter {
        match app.view {
            View::Entities => {
                if let Some(entity) = app.get_selected_entity().cloned() {
                    let logical_name = entity.logical_name.clone();
                    app.enter_entity_detail();
                    app.load_entity_detail(&logical_name).await;
                }
            }
            View::Users => {
                if let Some(user) = app.get_selected_user().cloned() {
                    let user_id = user.id.clone();
                    app.enter_user_detail();
                    app.load_user_detail(&user_id).await;
                }
            }
            View::Solutions => {
                if let Some(solution) = app.get_selected_solution().cloned() {
                    let solution_id = solution.solution_id.clone();
                    app.enter_solution_detail();
                    app.load_solution_detail(&solution_id).await;
                }
            }
            View::SolutionDetail => {
                app.jump_to_component().await;
            }
            View::RecordDetail => {
                app.navigate_to_related_record().await;
            }
            _ => {}
        }
    }

    // Tab key for switching tabs
    if key == KeyCode::Tab {
        app.next_tab();
    }
    if key == KeyCode::BackTab {
        app.prev_tab();
    }
}

/// Handle input in search mode
fn handle_search_mode(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Enter => {
            app.input_mode = InputMode::Normal;
            // Apply filter based on current view
            match app.view {
                View::Entities => app.filter_entities(),
                View::EntityDetail => app.filter_attributes(),
                View::Solutions => app.filter_solutions(),
                View::SolutionDetail => app.filter_solution_components(),
                View::Users => app.filter_users(),
                _ => {}
            }
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.search_query.clear();
            // Reset filters
            match app.view {
                View::Entities => app.filter_entities(),
                View::EntityDetail => app.filter_attributes(),
                View::Solutions => app.filter_solutions(),
                View::SolutionDetail => app.filter_solution_components(),
                View::Users => app.filter_users(),
                _ => {}
            }
        }
        KeyCode::Backspace => {
            app.search_query.pop();
        }
        KeyCode::Char(c) => {
            app.search_query.push(c);
        }
        _ => {}
    }
}

