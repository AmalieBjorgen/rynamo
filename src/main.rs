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
mod export;
mod config;

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

    // Load configuration
    let mut config = config::Config::load().unwrap_or_default();
    if !config.environments.contains(&args.env) {
        config.add_environment(args.env.clone());
        let _ = config.save();
    }

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
    app.config = config;
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
        if app.should_load_more_jobs {
            app.should_load_more_jobs = false;
            app.load_more_system_jobs().await;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                // Clear any global message on any keypress
                if app.message.is_some() {
                    app.clear_message();
                }

                match app.input_mode {
                    InputMode::Normal => handle_normal_mode(app, key.code).await?,
                    InputMode::Search => handle_search_mode(app, key.code).await?,
                    InputMode::FetchXML => handle_fetchxml_mode(app, key.code).await?,
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
async fn handle_normal_mode(app: &mut App, key: KeyCode) -> Result<()> {
    use crate::ui::EntityTab;
    
    // Global shortcuts
    match key {
        KeyCode::Char('q') => {
            // Only quit from main views, go back from detail views
            match app.view {
                View::EntityDetail | View::SolutionDetail | View::UserDetail => app.go_back(),
                _ => app.should_quit = true,
            }
            return Ok(());
        }
        KeyCode::Char('/') => {
            app.input_mode = InputMode::Search;
            app.search_query.clear();
            return Ok(());
        }
        KeyCode::Esc => {
            app.go_back();
            return Ok(());
        }
        KeyCode::Char('1') => {
            if app.view != View::Entities && app.view != View::EntityDetail {
                app.view = View::Entities;
            }
            return Ok(());
        }
        KeyCode::Char('2') => {
            if app.view != View::Solutions && app.view != View::SolutionDetail {
                app.view = View::Solutions;
                if app.solutions.is_empty() {
                    app.load_solutions().await;
                }
            }
            return Ok(());
        }
        KeyCode::Char('3') => {
            if app.view != View::Users && app.view != View::UserDetail {
                app.view = View::Users;
                if app.users.is_empty() {
                    app.load_users().await;
                }
            }
            return Ok(());
        }
        KeyCode::Char('4') => {
            if app.view != View::OptionSets {
                app.view = View::OptionSets;
                if app.global_optionsets.is_empty() {
                    app.load_global_optionsets().await;
                }
            }
            return Ok(());
        }
        KeyCode::Char('5') => {
            if app.view != View::SystemJobs && app.view != View::SystemJobDetail {
                app.view = View::SystemJobs;
                if app.system_jobs.is_empty() {
                    app.load_system_jobs(None).await;
                }
            }
            return Ok(());
        }
        KeyCode::Char('g') => {
            app.input_mode = InputMode::Search;
            app.search_query.clear();
            app.view = View::GlobalSearch;
            return Ok(());
        }
        KeyCode::Char('E') => {
            app.view = View::Environments;
            return Ok(());
        }
        KeyCode::Char('D') => {
            if app.view == View::Environments {
                app.discover_environments().await?;
            }
            return Ok(());
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            match app.view {
                View::Entities => {
                    if let Some(entity) = app.get_selected_entity() {
                        let metadata_id = entity.metadata_id.clone();
                        app.load_solution_layers(&metadata_id, 1).await;
                    }
                }
                View::EntityDetail => {
                    if app.entity_tab == crate::ui::EntityTab::Attributes {
                        if let Some(attr) = app.get_selected_attribute() {
                            let metadata_id = attr.metadata_id.clone();
                            app.load_solution_layers(&metadata_id, 2).await;
                        }
                    }
                }
                View::SolutionDetail => {
                    if let Some(comp) = app.get_selected_component() {
                        let type_code = comp.component_type.unwrap_or(0);
                        let object_id = comp.object_id.as_deref().unwrap_or(&comp.solution_component_id).to_string();
                        app.load_solution_layers(&object_id, type_code).await;
                    }
                }
                _ => {}
            }
            return Ok(());
        }
        KeyCode::Char('f') | KeyCode::Char('F') => {
            app.view = View::FetchXML;
            app.input_mode = crate::ui::InputMode::FetchXML;
            return Ok(());
        }
        _ => {}
    }

    // Navigation
    if app.key_bindings.is_up(key) {
        app.navigate_up();
        return Ok(());
    }
    if app.key_bindings.is_down(key) {
        app.navigate_down();
        return Ok(());
    }
    if app.key_bindings.is_left(key) {
        app.prev_tab();
        return Ok(());
    }
    if app.key_bindings.is_right(key) {
        app.next_tab();
        return Ok(());
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
                        'e' => {
                            if app.query_mode == crate::ui::QueryMode::Results {
                                app.export_query_results();
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
        return Ok(());
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
            View::OptionSets => {
                // No action for now
            }
            View::GlobalSearch => {
                app.enter_search_result().await;
            }
            View::Environments => {
                if let Some(url) = app.config.environments.get(app.environment_index).cloned() {
                    let _ = app.switch_environment(&url).await;
                }
            }
            View::SolutionDetail => {
                app.jump_to_component().await;
            }
            View::RecordDetail => {
                app.navigate_to_related_record().await;
            }
            View::SystemJobs => {
                 if !app.filtered_system_jobs.is_empty() {
                    let index = app.filtered_system_jobs[app.system_job_index];
                    app.selected_system_job = Some(app.system_jobs[index].clone());
                    
                    // Fetch details to get full message
                    if let Some(job) = &app.selected_system_job {
                         let id = job.id.clone();
                         if let Ok(details) = app.client.get_system_job(&id).await {
                             app.selected_system_job = Some(details);
                         }
                    }
                    
                    app.view = View::SystemJobDetail;
                }
            }
            _ => {}
        }
    }

    // Refresh for System Jobs
    if app.view == View::SystemJobs {
        if key == KeyCode::Char('r') || key == KeyCode::Char('R') {
            app.refresh_system_jobs().await;
            return Ok(());
        }
    }

    // Tab key for switching tabs
    if key == KeyCode::Tab {
        app.next_tab();
    }
    if key == KeyCode::BackTab {
        app.prev_tab();
    }

    Ok(())
}

/// Handle input in search mode
async fn handle_search_mode(app: &mut App, key: KeyCode) -> Result<()> {
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
                View::OptionSets => app.filter_optionsets(),
                View::SystemJobs => app.search_system_jobs().await,
                View::GlobalSearch => app.execute_global_search(),
                _ => {}
            }
        }
        KeyCode::Esc => {
             // ... existing code ...
             // Reset filters
            app.input_mode = InputMode::Normal;
            app.search_query.clear();
            match app.view {
                View::Entities => app.filter_entities(),
                View::EntityDetail => app.filter_attributes(),
                View::Solutions => app.filter_solutions(),
                View::SolutionDetail => app.filter_solution_components(),
                View::Users => app.filter_users(),
                View::OptionSets => app.filter_optionsets(),
                View::SystemJobs => app.load_system_jobs(None).await,
                View::GlobalSearch => app.execute_global_search(),
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
    Ok(())
}

/// Handle input in FetchXML mode
async fn handle_fetchxml_mode(app: &mut crate::ui::App, key: KeyCode) -> Result<()> {
    match key {
        KeyCode::Enter => {
            app.execute_fetch_xml_query().await;
            app.input_mode = crate::ui::InputMode::Normal;
        }
        KeyCode::Char(c) => {
            app.fetchxml_query.insert(app.fetchxml_cursor, c);
            app.fetchxml_cursor += 1;
        }
        KeyCode::Backspace => {
            if app.fetchxml_cursor > 0 {
                app.fetchxml_query.remove(app.fetchxml_cursor - 1);
                app.fetchxml_cursor -= 1;
            }
        }
        KeyCode::Left => {
            if app.fetchxml_cursor > 0 {
                app.fetchxml_cursor -= 1;
            }
        }
        KeyCode::Right => {
            if app.fetchxml_cursor < app.fetchxml_query.len() {
                app.fetchxml_cursor += 1;
            }
        }
        KeyCode::Esc => {
            app.input_mode = crate::ui::InputMode::Normal;
            app.view = crate::ui::View::Entities; // Default back
        }
        _ => {}
    }
    Ok(())
}

