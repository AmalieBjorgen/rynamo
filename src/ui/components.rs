//! UI rendering components

use ratatui::prelude::{Constraint, Direction, Layout, Rect, Line, Span, Modifier, Position};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Row, Table, TableState, Tabs, Wrap};
use ratatui::Frame;

use super::app::{App, AppState, EntityTab, QueryMode, SearchResult, UserTab, View};
use super::input::InputMode;
use crate::models::{ComponentType, RoleSource};

/// Render the complete UI
pub fn render(frame: &mut Frame, app: &mut App) {
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
    let titles = vec!["Entities [1]", "Solutions [2]", "Users [3]", "Choices [4]", "Search [G]", "Env [E]"];
    let selected = match app.view {
        View::Entities | View::EntityDetail | View::RecordDetail => 0,
        View::Solutions | View::SolutionDetail => 1,
        View::Users | View::UserDetail => 2,
        View::OptionSets => 3,
        View::GlobalSearch => 4,
        View::Environments => 5,
        View::SolutionLayers => 0,
        View::FetchXML => 0,
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
fn render_content(frame: &mut Frame, app: &mut App, area: Rect) {
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
            View::Users => render_user_list(frame, app, area),
            View::UserDetail => render_user_detail(frame, app, area),
            View::RecordDetail => render_record_detail(frame, app, area),
            View::OptionSets => render_optionset_browser(frame, app, area),
            View::GlobalSearch => render_global_search(frame, app, area),
            View::Environments => render_environment_switcher(frame, app, area),
            View::SolutionLayers => render_solution_layers(frame, app, area),
            View::FetchXML => render_fetchxml_console(frame, app, area),
        },
    }
}

/// Render the entity list view
fn render_entity_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_entities
        .iter()
        .map(|&entity_idx| {
            let entity = &app.entities[entity_idx];
            let is_custom = entity.is_custom_entity.unwrap_or(false);
            let prefix = if is_custom { "⚙ " } else { "  " };
            
            let content = format!(
                "{}{:<40} {}",
                prefix,
                entity.logical_name,
                entity.get_display_name()
            );

            let style = if is_custom {
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

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_bottom(" ↑↓ Navigate │ Enter: Details │ /: Search │ q: Quit "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.entity_index));

    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Render entity detail view
fn render_entity_detail(frame: &mut Frame, app: &mut App, area: Rect) {
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
        "Query".to_string(),
    ];
    let selected_tab = match app.entity_tab {
        EntityTab::Attributes => 0,
        EntityTab::Relationships => 1,
        EntityTab::Metadata => 2,
        EntityTab::Query => 3,
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
        EntityTab::Query => render_query_tab(frame, app, chunks[2]),
    }
}

/// Render attributes table
fn render_attributes(frame: &mut Frame, app: &mut App, area: Rect) {
    let header = Row::new(vec!["Logical Name", "Display Name", "Type", "Required"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .filtered_attributes
        .iter()
        .map(|&attr_idx| {
            let attr = &app.entity_attributes[attr_idx];
            let required = if attr.is_required() { "Yes" } else { "No" };

            Row::new(vec![
                attr.logical_name.clone(),
                attr.get_display_name(),
                attr.get_type_name(),
                required.to_string(),
            ])
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
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(50, 50, 80))
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    let mut table_state = TableState::default();
    table_state.select(Some(app.attribute_index));

    frame.render_stateful_widget(table, area, &mut table_state);
}

/// Render relationships list
fn render_relationships(frame: &mut Frame, app: &mut App, area: Rect) {
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

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Relationships ")
                .title_bottom(" ←→ Tabs │ Esc: Back "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .add_modifier(Modifier::BOLD),
        );

    let mut list_state = ListState::default();
    list_state.select(Some(app.relationship_index));

    frame.render_stateful_widget(list, area, &mut list_state);
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
fn render_solution_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_solutions
        .iter()
        .map(|&sol_idx| {
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

            ListItem::new(content)
        })
        .collect();

    let title = format!(
        " Solutions ({}/{}) ",
        app.filtered_solutions.len(),
        app.solutions.len()
    );

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_bottom(" 🔒 Managed │ 📝 Unmanaged │ /: Search │ q: Quit "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.solution_index));

    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Render solution detail
fn render_solution_detail(frame: &mut Frame, app: &App, area: Rect) {
    let Some(solution) = &app.selected_solution else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Solution info
            Constraint::Length(3), // Statistics summary / Search bar
            Constraint::Min(0),    // Components list
        ])
        .split(area);

    // 1. Solution Information
    let info_items = vec![
        format!("Friendly Name: {}", solution.get_display_name()),
        format!("Unique Name:   {}", solution.unique_name),
        format!("Version:       {}", solution.version.as_deref().unwrap_or("-")),
        format!("Managed:       {}", if solution.is_managed.unwrap_or(false) { "Yes" } else { "No" }),
        format!("Publisher:     {}", solution.publisher_id.as_deref().unwrap_or("-")),
    ];
    let info = Paragraph::new(info_items.join("\n"))
        .block(Block::default().borders(Borders::ALL).title(" Solution Information "));
    frame.render_widget(info, chunks[0]);

    // 2. Statistics & Search
    let mut stats = std::collections::HashMap::new();
    for comp in &app.solution_components {
        *stats.entry(comp.get_component_type()).or_insert(0) += 1;
    }
    let mut sorted_stats: Vec<_> = stats.into_iter().collect();
    sorted_stats.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    let stats_str = sorted_stats.iter()
        .take(5)
        .map(|(t, c)| format!("{}: {}", t.display_name(), c))
        .collect::<Vec<_>>()
        .join("  |  ");

    let search_text = if app.input_mode == InputMode::Search && app.view == View::SolutionDetail {
        format!("Search: {}_ (Type to filter)", app.search_query)
    } else if !app.search_query.is_empty() {
        format!("Filter: {} (Press '/' to search, Esc to clear)", app.search_query)
    } else {
        format!("Summary: {} (Press '/' to search)", stats_str)
    };

    let stats_para = Paragraph::new(search_text)
        .block(Block::default().borders(Borders::ALL).title(" Components Summary "));
    frame.render_widget(stats_para, chunks[1]);

    // 3. Components List
    let items: Vec<ListItem> = app.filtered_components
        .iter()
        .map(|&idx| {
            let comp = &app.solution_components[idx];
            let type_name = comp.get_component_type().display_name();
            let object_id = comp.object_id.as_deref().unwrap_or("-");
            
            // Try to resolve name if it's an entity
            let mut resolved_name = String::new();
            if comp.get_component_type() == ComponentType::Entity {
                if let Some(entity) = app.entities.iter().find(|e| e.metadata_id.to_lowercase() == object_id.to_lowercase()) {
                    resolved_name = format!(" [{}]", entity.get_display_name());
                }
            }

            let mut style = Style::default();
            if comp.get_component_type() == ComponentType::Entity {
                style = style.fg(Color::Cyan);
            }

            let content = format!("{:<20} {}{}", type_name, object_id, resolved_name);
            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(
        " Components (Showing {} of {}) - Enter: Drill Down / Esc: Back ",
        app.filtered_components.len(),
        app.solution_components.len()
    );
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    if !app.filtered_components.is_empty() {
        list_state.select(Some(app.component_index));
    }
    frame.render_stateful_widget(list, chunks[2], &mut list_state);
}

/// Render user list
fn render_user_list(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app
        .filtered_users
        .iter()
        .map(|&user_idx| {
            let user = &app.users[user_idx];
            let status = if user.is_disabled.unwrap_or(false) {
                "⊘"
            } else {
                "●"
            };

            let content = format!(
                "{} {:<35} {}",
                status,
                user.get_display_name(),
                user.email.as_deref().unwrap_or("")
            );

            let style = if user.is_disabled.unwrap_or(false) {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default()
            };

            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(
        " Users ({}/{}) ",
        app.filtered_users.len(),
        app.users.len()
    );

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_bottom(" ↑↓ Navigate │ Enter: Details │ /: Search │ q: Quit "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.user_index));

    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Render user detail view
fn render_user_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    let Some(user) = &app.selected_user else {
        return;
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // User info
            Constraint::Length(3), // Tabs
            Constraint::Min(0),    // Content
        ])
        .split(area);

    // User header
    let header = Paragraph::new(format!(
        "{} <{}>",
        user.get_display_name(),
        user.email.as_deref().unwrap_or("-")
    ))
    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(header, chunks[0]);

    // Tabs for user details
    let tab_titles = vec![
        format!("Direct Roles ({})", app.user_direct_roles.len()),
        format!("Teams ({})", app.user_teams.len()),
        format!("All Roles ({})", app.user_all_roles.len()),
        "Info".to_string(),
    ];
    let selected_tab = match app.user_tab {
        UserTab::DirectRoles => 0,
        UserTab::Teams => 1,
        UserTab::AllRoles => 2,
        UserTab::Info => 3,
    };

    let tabs = Tabs::new(tab_titles)
        .block(Block::default().borders(Borders::ALL))
        .select(selected_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD));
    frame.render_widget(tabs, chunks[1]);

    // Tab content
    match app.user_tab {
        UserTab::DirectRoles => render_user_direct_roles(frame, app, chunks[2]),
        UserTab::Teams => render_user_teams(frame, app, chunks[2]),
        UserTab::AllRoles => render_user_all_roles(frame, app, chunks[2]),
        UserTab::Info => render_user_info(frame, app, chunks[2]),
    }
}

/// Render direct roles table
fn render_user_direct_roles(frame: &mut Frame, app: &mut App, area: Rect) {
    let header = Row::new(vec!["Role Name", "Business Unit", "Managed"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .user_direct_roles
        .iter()
        .map(|role| {
            let managed = if role.is_managed.unwrap_or(false) { "Yes" } else { "No" };
            Row::new(vec![
                role.name.clone(),
                role.get_business_unit_name(),
                managed.to_string(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(35),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Direct Roles ({}) ", app.user_direct_roles.len()))
            .title_bottom(" ←→ Tabs │ Esc: Back "),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(50, 50, 80))
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    let mut table_state = TableState::default();
    table_state.select(Some(app.user_role_index));

    frame.render_stateful_widget(table, area, &mut table_state);
}

/// Render user teams
fn render_user_teams(frame: &mut Frame, app: &mut App, area: Rect) {
    let header = Row::new(vec!["Team Name", "Type", "Default"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .user_teams
        .iter()
        .map(|team| {
            let is_default = if team.is_default.unwrap_or(false) { "Yes" } else { "No" };
            Row::new(vec![
                team.name.clone(),
                team.get_type_name().to_string(),
                is_default.to_string(),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(35),
            Constraint::Percentage(15),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" Teams ({}) ", app.user_teams.len()))
            .title_bottom(" ←→ Tabs │ Esc: Back "),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(50, 50, 80))
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    let mut table_state = TableState::default();
    table_state.select(Some(app.user_team_index));

    frame.render_stateful_widget(table, area, &mut table_state);
}

/// Render all roles (direct + inherited)
fn render_user_all_roles(frame: &mut Frame, app: &mut App, area: Rect) {
    let header = Row::new(vec!["Role Name", "Source", "Business Unit"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app
        .user_all_roles
        .iter()
        .map(|assignment| {
            let source = match &assignment.source {
                RoleSource::Direct => "Direct".to_string(),
                RoleSource::Team(team_name) => format!("Team: {}", team_name),
            };
            
            let style = match &assignment.source {
                RoleSource::Direct => Style::default().fg(Color::Green),
                RoleSource::Team(_) => Style::default().fg(Color::Blue),
            };

            Row::new(vec![
                assignment.role.name.clone(),
                source,
                assignment.role.get_business_unit_name(),
            ]).style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(40),
            Constraint::Percentage(35),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!(" All Roles ({}) ", app.user_all_roles.len()))
            .title_bottom(" 🟢 Direct │ 🔵 Team │ ←→ Tabs │ Esc: Back "),
    )
    .row_highlight_style(
        Style::default()
            .bg(Color::Rgb(50, 50, 80))
            .add_modifier(Modifier::BOLD),
    )
    .highlight_symbol("▶ ");

    let mut table_state = TableState::default();
    table_state.select(Some(app.user_role_index));

    frame.render_stateful_widget(table, area, &mut table_state);
}

/// Render user info
fn render_user_info(frame: &mut Frame, app: &App, area: Rect) {
    let Some(user) = &app.selected_user else {
        return;
    };

    let bu_name = user.business_unit
        .as_ref()
        .and_then(|bu| bu.name.clone())
        .unwrap_or_else(|| "-".to_string());

    let info = vec![
        format!("Full Name:       {}", user.get_display_name()),
        format!("Domain Name:     {}", user.domain_name.as_deref().unwrap_or("-")),
        format!("Email:           {}", user.email.as_deref().unwrap_or("-")),
        format!("Title:           {}", user.title.as_deref().unwrap_or("-")),
        format!("Business Unit:   {}", bu_name),
        format!("Status:          {}", user.get_status()),
        format!("Created On:      {}", user.created_on.as_deref().unwrap_or("-")),
        String::new(),
        format!("Direct Roles:    {}", app.user_direct_roles.len()),
        format!("Teams:           {}", app.user_teams.len()),
        format!("Total Roles:     {} (including team roles)", app.user_all_roles.len()),
    ];

    let text: Vec<Line> = info.into_iter().map(Line::from).collect();

    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" User Info ")
                .title_bottom(" ←→ Tabs │ Esc: Back "),
        )
        .wrap(Wrap { trim: false });

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

    let message_text = if let Some(msg) = &app.message {
        format!(" │ {} ", msg)
    } else {
        String::new()
    };

    let status = Line::from(vec![
        state_indicator,
        Span::raw(format!("│ {} ", env)),
        Span::styled(search_hint, Style::default().fg(Color::Magenta)),
        Span::styled(message_text, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
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

/// Render Query tab in entity detail - guided query builder
fn render_query_tab(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // Left panel - columns/filters
            Constraint::Percentage(60), // Right panel - results
        ])
        .split(area);

    render_query_builder(frame, app, chunks[0]);
    render_query_results(frame, app, chunks[1]);
}

/// Render the query builder panels (columns, filters)
fn render_query_builder(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Mode tabs
            Constraint::Min(0),    // Content
            Constraint::Length(3), // Help
        ])
        .split(area);

    // Mode tabs
    let mode_titles = vec!["Columns", "Filters", "Options", "Results"];
    let selected_mode = match app.query_mode {
        QueryMode::Columns => 0,
        QueryMode::Filter => 1,
        QueryMode::OrderBy | QueryMode::Options => 2,
        QueryMode::Results => 3,
    };

    let mode_tabs = Tabs::new(mode_titles)
        .block(Block::default().borders(Borders::ALL))
        .select(selected_mode)
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    frame.render_widget(mode_tabs, chunks[0]);

    // Content based on mode
    match app.query_mode {
        QueryMode::Columns => render_column_selector(frame, app, chunks[1]),
        QueryMode::Filter => render_filter_builder(frame, app, chunks[1]),
        QueryMode::OrderBy | QueryMode::Options => render_query_options(frame, app, chunks[1]),
        QueryMode::Results => {
            // In results mode, show selected columns as a reference on the left
            render_column_selector(frame, app, chunks[1]);
        }
    }

    // Help text
    let help = match app.query_mode {
        QueryMode::Columns => "Tab: Next │ Enter: Filter by │ Space: Toggle │ a: All │ c: Clear │ F5: Run",
        QueryMode::Filter => "Tab: Next │ Enter: Add │ d: Delete │ o/O: Op │ Backspace: Pop │ F5: Run",
        QueryMode::Options | QueryMode::OrderBy => "Tab: Next │ Enter: Edit │ F5: Run",
        QueryMode::Results => "Tab: Next │ n: Next Page │ ↑/↓: Scroll │ Esc: Back │ F5: Run again",
    };
    let help_para = Paragraph::new(help)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help_para, chunks[2]);
}

/// Render column selector list
fn render_column_selector(frame: &mut Frame, app: &App, area: Rect) {
    let selected_count = app.query_selected_columns.iter().filter(|&&s| s).count();

    let items: Vec<ListItem> = app.entity_attributes
        .iter()
        .enumerate()
        .map(|(i, attr)| {
            let is_selected = app.query_selected_columns.get(i).copied().unwrap_or(false);
            let checkbox = if is_selected { "[✓]" } else { "[ ]" };
            let content = format!("{} {} ({})", checkbox, attr.logical_name, attr.get_type_name());
            
            let style = if is_selected {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            
            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(" Select Columns ({} selected) ", selected_count);
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.query_column_index));

    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Render filter builder
fn render_filter_builder(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Applied filters
            Constraint::Length(5), // New filter input
        ])
        .split(area);

    // Applied filters list
    let filter_items: Vec<ListItem> = app.query_filters
        .iter()
        .map(|f| {
            let content = format!("{} {} {}", f.attribute_name, f.operator.label(), f.value);
            ListItem::new(content).style(Style::default().fg(Color::Yellow))
        })
        .collect();

    let filter_title = format!(" Filters ({}) ", app.query_filters.len());
    let filter_list = List::new(filter_items)
        .block(Block::default().borders(Borders::ALL).title(filter_title))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut filter_state = ListState::default();
    if !app.query_filters.is_empty() {
        filter_state.select(Some(app.query_filter_index));
    }
    frame.render_stateful_widget(filter_list, chunks[0], &mut filter_state);

    // New filter input
    let new_filter_text = if let Some(attr_idx) = app.query_filter_attr {
        let attr_name = app.entity_attributes.get(attr_idx)
            .map(|a| a.logical_name.as_str())
            .unwrap_or("?");
        format!("{} {} {}", attr_name, app.query_filter_op.label(), app.query_filter_value)
    } else {
        "Press Enter on a column to add filter".to_string()
    };
    
    let new_filter = Paragraph::new(new_filter_text)
        .style(Style::default().fg(Color::Cyan))
        .block(Block::default().borders(Borders::ALL).title(" New Filter "));
    frame.render_widget(new_filter, chunks[1]);
}

/// Render query options (order by, top)
fn render_query_options(frame: &mut Frame, app: &App, area: Rect) {
    let order_by_text = match app.query_order_by {
        Some(idx) => {
            let name = app.entity_attributes.get(idx)
                .map(|a| a.logical_name.as_str())
                .unwrap_or("?");
            let dir = if app.query_order_desc { " DESC" } else { " ASC" };
            format!("{}{}", name, dir)
        }
        None => "(none)".to_string(),
    };

    let top_text = app.query_top
        .map(|n| n.to_string())
        .unwrap_or_else(|| "(all)".to_string());

    let options = vec![
        format!("Order By:  {}", order_by_text),
        format!("Top:       {}", top_text),
    ];

    let text: Vec<Line> = options.into_iter().map(Line::from).collect();
    let paragraph = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title(" Options "));
    frame.render_widget(paragraph, area);
}

/// Render query results
fn render_query_results(frame: &mut Frame, app: &mut App, area: Rect) {
    // Check for error
    if let Some(ref error) = app.query_result.error {
        let paragraph = Paragraph::new(error.as_str())
            .style(Style::default().fg(Color::Red))
            .block(Block::default().borders(Borders::ALL).title(" Error "))
            .wrap(Wrap { trim: true });
        frame.render_widget(paragraph, area);
        return;
    }

    // Check if we have results
    if app.query_result.columns.is_empty() {
        let paragraph = Paragraph::new("Press F5 to run query")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL).title(" Results "));
        frame.render_widget(paragraph, area);
        return;
    }

    // Render results table
    let header = Row::new(app.query_result.columns.clone())
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = app.query_result.rows.iter().map(|row| {
        Row::new(row.clone())
    }).collect();

    // Calculate column widths (equal distribution)
    let col_count = app.query_result.columns.len();
    let widths: Vec<Constraint> = if col_count > 0 {
        vec![Constraint::Percentage((100 / col_count as u16).max(1)); col_count]
    } else {
        vec![]
    };

    let has_more = app.query_result.next_link.is_some();
    let title = format!(
        " Results ({} rows){} {} ",
        app.query_result.rows.len(),
        if has_more { " [Press 'n' for more]" } else { "" },
        if app.query_mode == QueryMode::Results { "[ACTIVE]" } else { "" }
    );

    let border_style = if app.query_mode == QueryMode::Results {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .border_style(border_style),
        )
        .row_highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut table_state = TableState::default();
    table_state.select(Some(app.query_result_index));

    frame.render_stateful_widget(table, area, &mut table_state);
}

/// Render record detail view
fn render_record_detail(frame: &mut Frame, app: &mut App, area: Rect) {
    let Some(row_idx) = app.selected_record_index else {
        return;
    };
    let Some(row) = app.query_result.rows.get(row_idx) else {
        return;
    };

    let items: Vec<ListItem> = app.query_result.columns
        .iter()
        .enumerate()
        .map(|(col_idx, col)| {
            let val = row.get(col_idx).cloned().unwrap_or_default();
            let mut style = Style::default();
            let mut content = format!("{:<30}: {}", col, val);
            
            if app.query_result.lookups.contains_key(&(row_idx, col_idx)) {
                content.push_str(" [Lookup ↵]");
                style = style.fg(Color::Cyan);
            }
            
            ListItem::new(content).style(style)
        })
        .collect();

    let title = format!(" Record Details [Row {}] ", row_idx + 1);
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(title)
                .title_bottom(" Esc: Back │ Enter: Navigate │ ↑↓: Scroll "),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    list_state.select(Some(app.record_detail_index));
    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Render solution layers for a component
fn render_solution_layers(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app.solution_layers
        .iter()
        .map(|layer| {
            let style = if layer.is_managed {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            };
            
            let managed_tag = if layer.is_managed { "[Managed]" } else { "[Unmanaged]" };
            
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:<2} ", layer.order), Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:<12} ", managed_tag), style),
                Span::styled(&layer.solution_name, Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(format!(" ({})", layer.name), Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let title = " Solution Layers (Top to Bottom) - Esc: Back ";
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    if !app.solution_layers.is_empty() {
        list_state.select(Some(app.solution_layers_index));
    }
    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Render global option set browser
fn render_optionset_browser(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(40), // OptionSet list
            Constraint::Percentage(60), // Selected values
        ])
        .split(area);

    // 1. OptionSet List
    let items: Vec<ListItem> = app.filtered_optionsets
        .iter()
        .map(|&idx| {
            let os = &app.global_optionsets[idx];
            let name = os.get_display_name();
            let sub = format!(" ({})", os.name);
            ListItem::new(Line::from(vec![
                Span::styled(name, Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(sub, Style::default().fg(Color::DarkGray)),
            ]))
        })
        .collect();

    let title = format!(" Global Choices ({}) ", app.filtered_optionsets.len());
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    if !app.filtered_optionsets.is_empty() {
        list_state.select(Some(app.optionset_index));
    }
    frame.render_stateful_widget(list, chunks[0], &mut list_state);

    // 2. Selected OptionSet Values
    let selected_idx = app.filtered_optionsets.get(app.optionset_index);
    let detail_block = Block::default()
        .borders(Borders::ALL)
        .title(" Choice Values ");

    if let Some(&idx) = selected_idx {
        let os = &app.global_optionsets[idx];
        if let Some(options) = &os.options {
            let rows: Vec<Row> = options.iter()
                .map(|opt| {
                    Row::new(vec![
                        opt.value.to_string(),
                        opt.get_label(),
                    ])
                })
                .collect();

            let table = Table::new(
                rows,
                [Constraint::Length(12), Constraint::Min(0)]
            )
            .header(
                Row::new(vec!["Value", "Label"])
                    .style(Style::default().add_modifier(Modifier::BOLD))
                    .bottom_margin(1)
            )
            .block(detail_block);

            frame.render_widget(table, chunks[1]);
        } else {
            let msg = Paragraph::new("\n  No values available/loaded for this choice set.")
                .block(detail_block);
            frame.render_widget(msg, chunks[1]);
        }
    }
}

/// Render global search results
fn render_global_search(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app.global_search_results
        .iter()
        .map(|result| {
            match result {
                SearchResult::Entity(idx) => {
                    let entity = &app.entities[*idx];
                    ListItem::new(Line::from(vec![
                        Span::styled(" [Entity]   ", Style::default().fg(Color::Cyan)),
                        Span::styled(entity.get_display_name(), Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" ({})", entity.logical_name), Style::default().fg(Color::DarkGray)),
                    ]))
                }
                SearchResult::Solution(idx) => {
                    let solution = &app.solutions[*idx];
                    ListItem::new(Line::from(vec![
                        Span::styled(" [Solution] ", Style::default().fg(Color::Yellow)),
                        Span::styled(solution.get_display_name(), Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" ({})", solution.unique_name), Style::default().fg(Color::DarkGray)),
                    ]))
                }
                SearchResult::OptionSet(idx) => {
                    let os = &app.global_optionsets[*idx];
                    ListItem::new(Line::from(vec![
                        Span::styled(" [Choice]   ", Style::default().fg(Color::Green)),
                        Span::styled(os.get_display_name(), Style::default().add_modifier(Modifier::BOLD)),
                        Span::styled(format!(" ({})", os.name), Style::default().fg(Color::DarkGray)),
                    ]))
                }
            }
        })
        .collect();

    let title = format!(
        " Global Search: '{}' ({}) - Enter: Go To / Esc: Back ",
        app.search_query,
        app.global_search_results.len()
    );
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    if !app.global_search_results.is_empty() {
        list_state.select(Some(app.global_search_index));
    }
    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Render environment switcher
fn render_environment_switcher(frame: &mut Frame, app: &mut App, area: Rect) {
    let items: Vec<ListItem> = app.config.environments
        .iter()
        .map(|url| {
            let is_current = app.config.current_env.as_ref().map_or(false, |curr| curr == url);
            let prefix = if is_current { "● " } else { "○ " };
            let style = if is_current {
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(url, style),
            ]))
        })
        .collect();

    let title = " Switch Environment - Enter: Select / Esc: Back ";
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(50, 50, 80))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▶ ");

    let mut list_state = ListState::default();
    if !app.config.environments.is_empty() {
        list_state.select(Some(app.environment_index));
    }
    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Render FetchXML Console
fn render_fetchxml_console(frame: &mut Frame, app: &mut App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10), // Input area
            Constraint::Length(3), // Help area
        ])
        .split(area);

    let input = Paragraph::new(app.fetchxml_query.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" FetchXML Console ")
                .border_style(Style::default().fg(Color::Cyan)),
        );
    
    frame.render_widget(input, chunks[0]);
    
    // Set cursor position
    frame.set_cursor_position(Position::new(
        chunks[0].x + 1 + app.fetchxml_cursor as u16,
        chunks[0].y + 1,
    ));

    let help = Paragraph::new(" Enter: Execute │ Esc: Back │ Ctrl+V: Paste ")
        .block(Block::default().borders(Borders::ALL));
    
    frame.render_widget(help, chunks[1]);
}
