//! UI rendering components

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, Tabs, Wrap},
    Frame,
};

use super::app::{App, AppState, EntityTab, View};
use super::input::InputMode;

/// Render the complete UI
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header/tabs
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    render_header(frame, app, chunks[0]);
    render_content(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    // Render search popup if in search mode
    if app.input_mode == InputMode::Search {
        render_search_popup(frame, app);
    }
}

/// Render the header with navigation tabs
fn render_header(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["Entities [1]", "Solutions [2]"];
    let selected = match app.view {
        View::Entities | View::EntityDetail => 0,
        View::Solutions | View::SolutionDetail => 1,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Rynamo "))
        .select(selected)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

/// Render the main content area
fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.state {
        AppState::Loading => {
            let loading = Paragraph::new("Loading...")
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL));
            frame.render_widget(loading, area);
        }
        AppState::Error => {
            let error_msg = app.error.as_deref().unwrap_or("Unknown error");
            let error = Paragraph::new(error_msg)
                .style(Style::default().fg(Color::Red))
                .block(Block::default().borders(Borders::ALL).title(" Error "))
                .wrap(Wrap { trim: true });
            frame.render_widget(error, area);
        }
        AppState::Ready => match app.view {
            View::Entities => render_entity_list(frame, app, area),
            View::EntityDetail => render_entity_detail(frame, app, area),
            View::Solutions => render_solution_list(frame, app, area),
            View::SolutionDetail => render_solution_detail(frame, app, area),
        },
    }
}

/// Render the entity list view
fn render_entity_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_entities
        .iter()
        .enumerate()
        .map(|(idx, &entity_idx)| {
            let entity = &app.entities[entity_idx];
            let is_custom = entity.is_custom_entity.unwrap_or(false);
            let prefix = if is_custom { "⚙ " } else { "  " };
            
            let content = format!(
                "{}{:<40} {}",
                prefix,
                entity.logical_name,
                entity.get_display_name()
            );

            let style = if idx == app.entity_index {
                Style::default()
                    .bg(Color::Rgb(50, 50, 80))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else if is_custom {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(
        " Entities ({}/{}) ",
        app.filtered_entities.len(),
        app.entities.len()
    );

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_bottom(" ↑↓ Navigate │ Enter: Details │ /: Search │ q: Quit "),
    );

    frame.render_widget(list, area);
}

/// Render entity detail view
fn render_entity_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(entity) = &app.selected_entity else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Entity info
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // Entity header
    let header = Paragraph::new(format!(
        "{} ({})",
        entity.get_display_name(),
        entity.logical_name
    ))
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Tabs for details
    let tab_titles = vec![
        format!("Attributes ({})", app.entity_attributes.len()),
        format!(
            "Relationships ({})",
            app.one_to_many.len() + app.many_to_one.len() + app.many_to_many.len()
        ),
        "Metadata".to_string(),
    ];
    let selected_tab = match app.entity_tab {
        EntityTab::Attributes => 0,
        EntityTab::Relationships => 1,
        EntityTab::Metadata => 2,
    };

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL))
        .select(selected_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, chunks[1]);

    // Tab content
    match app.entity_tab {
        EntityTab::Attributes => render_attributes(frame, app, chunks[2]),
        EntityTab::Relationships => render_relationships(frame, app, chunks[2]),
        EntityTab::Metadata => render_entity_metadata(frame, app, chunks[2]),
    }
}

/// Render attributes table
fn render_attributes(frame: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Logical Name", "Display Name", "Type", "Required"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .filtered_attributes
        .iter()
        .enumerate()
        .map(|(idx, &attr_idx)| {
            let attr = &app.entity_attributes[attr_idx];
            let required = if attr.is_required() { "Yes" } else { "No" };
            
            let style = if idx == app.attribute_index {
                Style::default()
                    .bg(Color::Rgb(50, 50, 80))
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            Row::new(vec![
                attr.logical_name.clone(),
                attr.get_display_name(),
                attr.get_type_name(),
                required.to_string(),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(30),
            Constraint::Percentage(25),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(
                " Attributes ({}/{}) ",
                app.filtered_attributes.len(),
                app.entity_attributes.len()
            ))
            .title_bottom(" ←→ Tabs │ Esc: Back │ /: Search "),
    );

    frame.render_widget(table, area);
}

/// Render relationships list
fn render_relationships(frame: &mut Frame, app: &App, area: Rect) {
    let mut items: Vec<ListItem> = Vec::new();

    // 1:N relationships
    if !app.one_to_many.is_empty() {
        items.push(ListItem::new("── One-to-Many (1:N) ──").style(Style::default().fg(Color::Yellow)));
        for rel in &app.one_to_many {
            let content = format!(
                "  {} → {}",
                rel.get_name(),
                rel.referencing_entity.as_deref().unwrap_or("?")
            );
            items.push(ListItem::new(content));
        }
    }

    // N:1 relationships
    if !app.many_to_one.is_empty() {
        items.push(ListItem::new("── Many-to-One (N:1) ──").style(Style::default().fg(Color::Yellow)));
        for rel in &app.many_to_one {
            let content = format!(
                "  {} → {}",
                rel.get_name(),
                rel.referenced_entity.as_deref().unwrap_or("?")
            );
            items.push(ListItem::new(content));
        }
    }

    // N:N relationships
    if !app.many_to_many.is_empty() {
        items.push(ListItem::new("── Many-to-Many (N:N) ──").style(Style::default().fg(Color::Yellow)));
        for rel in &app.many_to_many {
            let related = app
                .selected_entity
                .as_ref()
                .and_then(|e| rel.get_related_entity(&e.logical_name))
                .unwrap_or_else(|| "?".to_string());
            let content = format!("  {} ↔ {}", rel.get_name(), related);
            items.push(ListItem::new(content));
        }
    }

    if items.is_empty() {
        items.push(ListItem::new("No relationships found").style(Style::default().fg(Color::DarkGray)));
    }

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Relationships ")
            .title_bottom(" ←→ Tabs │ Esc: Back "),
    );

    frame.render_widget(list, area);
}

/// Render entity metadata
fn render_entity_metadata(frame: &mut Frame, app: &App, area: Rect) {
    let Some(entity) = &app.selected_entity else {
        return;
    };

    let info = vec![
        format!("Logical Name:       {}", entity.logical_name),
        format!("Schema Name:        {}", entity.schema_name.as_deref().unwrap_or("-")),
        format!("Display Name:       {}", entity.get_display_name()),
        format!("Entity Set Name:    {}", entity.entity_set_name.as_deref().unwrap_or("-")),
        format!("Primary ID:         {}", entity.primary_id_attribute.as_deref().unwrap_or("-")),
        format!("Primary Name:       {}", entity.primary_name_attribute.as_deref().unwrap_or("-")),
        format!("Object Type Code:   {}", entity.object_type_code.map(|c| c.to_string()).unwrap_or("-".to_string())),
        format!("Is Custom:          {}", entity.is_custom_entity.map(|b| if b { "Yes" } else { "No" }).unwrap_or("-")),
        format!("Is Managed:         {}", entity.is_managed.map(|b| if b { "Yes" } else { "No" }).unwrap_or("-")),
        String::new(),
        "Description:".to_string(),
        entity.get_description(),
    ];

    let text: Vec<Line> = info.into_iter().map(Line::from).collect();

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Metadata ")
                .title_bottom(" ←→ Tabs │ Esc: Back "),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(paragraph, area);
}

/// Render solution list
fn render_solution_list(frame: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_solutions
        .iter()
        .enumerate()
        .map(|(idx, &sol_idx)| {
            let solution = &app.solutions[sol_idx];
            let managed = if solution.is_managed.unwrap_or(false) {
                "🔒"
            } else {
                "📝"
            };

            let content = format!(
                "{} {:<40} v{}",
                managed,
                solution.get_display_name(),
                solution.version.as_deref().unwrap_or("?")
            );

            let style = if idx == app.solution_index {
                Style::default()
                    .bg(Color::Rgb(50, 50, 80))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(
        " Solutions ({}/{}) ",
        app.filtered_solutions.len(),
        app.solutions.len()
    );

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(title)
            .title_bottom(" 🔒 Managed │ 📝 Unmanaged │ /: Search │ q: Quit "),
    );

    frame.render_widget(list, area);
}

/// Render solution detail (placeholder for now)
fn render_solution_detail(frame: &mut Frame, _app: &App, area: Rect) {
    let paragraph = Paragraph::new("Solution detail view - Coming soon!")
        .block(Block::default().borders(Borders::ALL).title(" Solution Details "));
    frame.render_widget(paragraph, area);
}

/// Render the status bar
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let env = app.client.environment_url();
    let state_indicator = match app.state {
        AppState::Loading => Span::styled(" ● Loading ", Style::default().fg(Color::Yellow)),
        AppState::Ready => Span::styled(" ● Connected ", Style::default().fg(Color::Green)),
        AppState::Error => Span::styled(" ● Error ", Style::default().fg(Color::Red)),
    };

    let search_hint = if !app.search_query.is_empty() {
        format!(" │ Filter: {} ", app.search_query)
    } else {
        String::new()
    };

    let status = Line::from(vec![
        state_indicator,
        Span::raw(format!("│ {} ", env)),
        Span::styled(search_hint, Style::default().fg(Color::Magenta)),
    ]);

    let paragraph = Paragraph::new(status).block(Block::default().borders(Borders::ALL));
    frame.render_widget(paragraph, area);
}

/// Render search popup
fn render_search_popup(frame: &mut Frame, app: &App) {
    let area = centered_rect(50, 3, frame.area());
    
    frame.render_widget(Clear, area);
    
    let input = Paragraph::new(app.search_query.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Search (Enter to apply, Esc to cancel) ")
                .style(Style::default().fg(Color::Cyan)),
        );
    
    frame.render_widget(input, area);
    
    // Show cursor
    frame.set_cursor_position((
        area.x + app.search_query.len() as u16 + 1,
        area.y + 1,
    ));
}

/// Create a centered rectangle
fn centered_rect(percent_x: u16, height: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length((r.height - height) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
