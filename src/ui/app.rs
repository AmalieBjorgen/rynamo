//! Application state and main TUI logic

use crate::api::DataverseClient;
use crate::models::{
    AttributeMetadata, EntityMetadata, QueryDefinition, QueryField, QueryResult,
    RelationshipMetadata, RoleAssignment, RoleSource, SecurityRole, Solution, SystemUser, Team,
};
use super::input::{InputMode, KeyBindings};
use std::sync::Arc;

/// Current view in the application
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Entities,
    EntityDetail,
    Solutions,
    SolutionDetail,
    Users,
    UserDetail,
    Query,
}

/// Application state for the TUI
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppState {
    #[default]
    Loading,
    Ready,
    Error,
}

/// Detail tab for entity view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EntityTab {
    #[default]
    Attributes,
    Relationships,
    Metadata,
}

/// Detail tab for user view
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UserTab {
    #[default]
    DirectRoles,
    Teams,
    AllRoles,
    Info,
}

/// Main application struct
pub struct App {
    /// Dataverse API client
    pub client: Arc<DataverseClient>,

    /// Current application state
    pub state: AppState,

    /// Current view
    pub view: View,

    /// Error message if any
    pub error: Option<String>,

    /// Key binding style
    pub key_bindings: KeyBindings,

    /// Input mode
    pub input_mode: InputMode,

    /// Search/filter query
    pub search_query: String,

    // Entity list state
    pub entities: Vec<EntityMetadata>,
    pub filtered_entities: Vec<usize>,
    pub entity_index: usize,

    // Entity detail state
    pub selected_entity: Option<EntityMetadata>,
    pub entity_attributes: Vec<AttributeMetadata>,
    pub filtered_attributes: Vec<usize>,
    pub attribute_index: usize,
    pub one_to_many: Vec<RelationshipMetadata>,
    pub many_to_one: Vec<RelationshipMetadata>,
    pub many_to_many: Vec<RelationshipMetadata>,
    pub entity_tab: EntityTab,
    pub relationship_index: usize,

    // Solution list state
    pub solutions: Vec<Solution>,
    pub filtered_solutions: Vec<usize>,
    pub solution_index: usize,

    // User list state
    pub users: Vec<SystemUser>,
    pub filtered_users: Vec<usize>,
    pub user_index: usize,
    pub show_disabled_users: bool,

    // User detail state
    pub selected_user: Option<SystemUser>,
    pub user_tab: UserTab,
    pub user_direct_roles: Vec<SecurityRole>,
    pub user_teams: Vec<Team>,
    pub user_all_roles: Vec<RoleAssignment>,
    pub user_role_index: usize,
    pub user_team_index: usize,

    // Query builder state
    pub query: QueryDefinition,
    pub query_field: QueryField,
    pub query_input: String,
    pub query_result: QueryResult,
    pub query_result_index: usize,
    pub query_editing: bool,

    /// Should quit
    pub should_quit: bool,
}

impl App {
    /// Create a new app instance
    pub fn new(client: Arc<DataverseClient>, key_bindings: KeyBindings) -> Self {
        Self {
            client,
            state: AppState::Loading,
            view: View::Entities,
            error: None,
            key_bindings,
            input_mode: InputMode::Normal,
            search_query: String::new(),
            entities: Vec::new(),
            filtered_entities: Vec::new(),
            entity_index: 0,
            selected_entity: None,
            entity_attributes: Vec::new(),
            filtered_attributes: Vec::new(),
            attribute_index: 0,
            one_to_many: Vec::new(),
            many_to_one: Vec::new(),
            many_to_many: Vec::new(),
            entity_tab: EntityTab::Attributes,
            relationship_index: 0,
            solutions: Vec::new(),
            filtered_solutions: Vec::new(),
            solution_index: 0,
            users: Vec::new(),
            filtered_users: Vec::new(),
            user_index: 0,
            show_disabled_users: false,
            selected_user: None,
            user_tab: UserTab::DirectRoles,
            user_direct_roles: Vec::new(),
            user_teams: Vec::new(),
            user_all_roles: Vec::new(),
            user_role_index: 0,
            user_team_index: 0,
            query: QueryDefinition::default(),
            query_field: QueryField::Entity,
            query_input: String::new(),
            query_result: QueryResult::default(),
            query_result_index: 0,
            query_editing: false,
            should_quit: false,
        }
    }

    /// Load initial data (entities)
    pub async fn load_entities(&mut self) {
        self.state = AppState::Loading;
        self.error = None;

        match self.client.get_entities().await {
            Ok(mut entities) => {
                entities.sort_by(|a, b| a.logical_name.cmp(&b.logical_name));
                self.filtered_entities = (0..entities.len()).collect();
                self.entities = entities;
                self.state = AppState::Ready;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load entities: {}", e));
                self.state = AppState::Error;
            }
        }
    }

    /// Load entity details
    pub async fn load_entity_detail(&mut self, logical_name: &str) {
        self.state = AppState::Loading;
        self.error = None;

        match self.client.get_entity_attributes(logical_name).await {
            Ok(mut attrs) => {
                attrs.sort_by(|a, b| a.logical_name.cmp(&b.logical_name));
                self.filtered_attributes = (0..attrs.len()).collect();
                self.entity_attributes = attrs;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load attributes: {}", e));
                self.state = AppState::Error;
                return;
            }
        }

        if let Ok(rels) = self.client.get_entity_one_to_many(logical_name).await {
            self.one_to_many = rels;
        }
        if let Ok(rels) = self.client.get_entity_many_to_one(logical_name).await {
            self.many_to_one = rels;
        }
        if let Ok(rels) = self.client.get_entity_many_to_many(logical_name).await {
            self.many_to_many = rels;
        }

        self.attribute_index = 0;
        self.relationship_index = 0;
        self.entity_tab = EntityTab::Attributes;
        self.state = AppState::Ready;
    }

    /// Load solutions
    pub async fn load_solutions(&mut self) {
        self.state = AppState::Loading;
        self.error = None;

        match self.client.get_solutions().await {
            Ok(solutions) => {
                self.filtered_solutions = (0..solutions.len()).collect();
                self.solutions = solutions;
                self.state = AppState::Ready;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load solutions: {}", e));
                self.state = AppState::Error;
            }
        }
    }

    /// Load users
    pub async fn load_users(&mut self) {
        self.state = AppState::Loading;
        self.error = None;

        let result = if self.show_disabled_users {
            self.client.get_all_users().await
        } else {
            self.client.get_users().await
        };

        match result {
            Ok(users) => {
                self.filtered_users = (0..users.len()).collect();
                self.users = users;
                self.state = AppState::Ready;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load users: {}", e));
                self.state = AppState::Error;
            }
        }
    }

    /// Load user details (roles, teams)
    pub async fn load_user_detail(&mut self, user_id: &str) {
        self.state = AppState::Loading;
        self.error = None;

        // Load direct roles
        match self.client.get_user_roles(user_id).await {
            Ok(roles) => {
                self.user_direct_roles = roles;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load user roles: {}", e));
                self.state = AppState::Error;
                return;
            }
        }

        // Load teams
        match self.client.get_user_teams(user_id).await {
            Ok(teams) => {
                self.user_teams = teams;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load user teams: {}", e));
                self.state = AppState::Error;
                return;
            }
        }

        // Build combined role list (direct + from teams)
        self.user_all_roles.clear();

        // Add direct roles
        for role in &self.user_direct_roles {
            self.user_all_roles.push(RoleAssignment {
                role: role.clone(),
                source: RoleSource::Direct,
            });
        }

        // Add team roles
        for team in &self.user_teams {
            if let Ok(team_roles) = self.client.get_team_roles(&team.id).await {
                for role in team_roles {
                    self.user_all_roles.push(RoleAssignment {
                        role,
                        source: RoleSource::Team(team.name.clone()),
                    });
                }
            }
        }

        // Sort by role name
        self.user_all_roles.sort_by(|a, b| a.role.name.cmp(&b.role.name));

        self.user_role_index = 0;
        self.user_team_index = 0;
        self.user_tab = UserTab::DirectRoles;
        self.state = AppState::Ready;
    }

    /// Apply search filter to entities
    pub fn filter_entities(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.filtered_entities = (0..self.entities.len()).collect();
        } else {
            self.filtered_entities = self
                .entities
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    e.logical_name.to_lowercase().contains(&query)
                        || e.get_display_name().to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.entity_index = 0;
    }

    /// Apply search filter to attributes
    pub fn filter_attributes(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.filtered_attributes = (0..self.entity_attributes.len()).collect();
        } else {
            self.filtered_attributes = self
                .entity_attributes
                .iter()
                .enumerate()
                .filter(|(_, a)| {
                    a.logical_name.to_lowercase().contains(&query)
                        || a.get_display_name().to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.attribute_index = 0;
    }

    /// Apply search filter to solutions
    pub fn filter_solutions(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.filtered_solutions = (0..self.solutions.len()).collect();
        } else {
            self.filtered_solutions = self
                .solutions
                .iter()
                .enumerate()
                .filter(|(_, s)| {
                    s.unique_name.to_lowercase().contains(&query)
                        || s.get_display_name().to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.solution_index = 0;
    }

    /// Apply search filter to users
    pub fn filter_users(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.filtered_users = (0..self.users.len()).collect();
        } else {
            self.filtered_users = self
                .users
                .iter()
                .enumerate()
                .filter(|(_, u)| {
                    u.get_display_name().to_lowercase().contains(&query)
                        || u.domain_name.as_ref().map(|d| d.to_lowercase().contains(&query)).unwrap_or(false)
                        || u.email.as_ref().map(|e| e.to_lowercase().contains(&query)).unwrap_or(false)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.user_index = 0;
    }

    /// Navigate up in the current list
    pub fn navigate_up(&mut self) {
        match self.view {
            View::Entities => {
                if self.entity_index > 0 {
                    self.entity_index -= 1;
                }
            }
            View::EntityDetail => match self.entity_tab {
                EntityTab::Attributes => {
                    if self.attribute_index > 0 {
                        self.attribute_index -= 1;
                    }
                }
                EntityTab::Relationships => {
                    if self.relationship_index > 0 {
                        self.relationship_index -= 1;
                    }
                }
                EntityTab::Metadata => {}
            },
            View::Solutions => {
                if self.solution_index > 0 {
                    self.solution_index -= 1;
                }
            }
            View::Users => {
                if self.user_index > 0 {
                    self.user_index -= 1;
                }
            }
            View::UserDetail => match self.user_tab {
                UserTab::DirectRoles | UserTab::AllRoles => {
                    if self.user_role_index > 0 {
                        self.user_role_index -= 1;
                    }
                }
                UserTab::Teams => {
                    if self.user_team_index > 0 {
                        self.user_team_index -= 1;
                    }
                }
                UserTab::Info => {}
            },
            View::SolutionDetail | View::Query => {}
        }
    }

    /// Navigate down in the current list
    pub fn navigate_down(&mut self) {
        match self.view {
            View::Entities => {
                if !self.filtered_entities.is_empty()
                    && self.entity_index < self.filtered_entities.len() - 1
                {
                    self.entity_index += 1;
                }
            }
            View::EntityDetail => match self.entity_tab {
                EntityTab::Attributes => {
                    if !self.filtered_attributes.is_empty()
                        && self.attribute_index < self.filtered_attributes.len() - 1
                    {
                        self.attribute_index += 1;
                    }
                }
                EntityTab::Relationships => {
                    let total = self.one_to_many.len() + self.many_to_one.len() + self.many_to_many.len();
                    if total > 0 && self.relationship_index < total - 1 {
                        self.relationship_index += 1;
                    }
                }
                EntityTab::Metadata => {}
            },
            View::Solutions => {
                if !self.filtered_solutions.is_empty()
                    && self.solution_index < self.filtered_solutions.len() - 1
                {
                    self.solution_index += 1;
                }
            }
            View::Users => {
                if !self.filtered_users.is_empty()
                    && self.user_index < self.filtered_users.len() - 1
                {
                    self.user_index += 1;
                }
            }
            View::UserDetail => match self.user_tab {
                UserTab::DirectRoles => {
                    if !self.user_direct_roles.is_empty()
                        && self.user_role_index < self.user_direct_roles.len() - 1
                    {
                        self.user_role_index += 1;
                    }
                }
                UserTab::AllRoles => {
                    if !self.user_all_roles.is_empty()
                        && self.user_role_index < self.user_all_roles.len() - 1
                    {
                        self.user_role_index += 1;
                    }
                }
                UserTab::Teams => {
                    if !self.user_teams.is_empty()
                        && self.user_team_index < self.user_teams.len() - 1
                    {
                        self.user_team_index += 1;
                    }
                }
                UserTab::Info => {}
            },
            View::SolutionDetail | View::Query => {}
        }
    }

    /// Navigate to next tab
    pub fn next_tab(&mut self) {
        match self.view {
            View::EntityDetail => {
                self.entity_tab = match self.entity_tab {
                    EntityTab::Attributes => EntityTab::Relationships,
                    EntityTab::Relationships => EntityTab::Metadata,
                    EntityTab::Metadata => EntityTab::Attributes,
                };
            }
            View::UserDetail => {
                self.user_tab = match self.user_tab {
                    UserTab::DirectRoles => UserTab::Teams,
                    UserTab::Teams => UserTab::AllRoles,
                    UserTab::AllRoles => UserTab::Info,
                    UserTab::Info => UserTab::DirectRoles,
                };
                self.user_role_index = 0;
                self.user_team_index = 0;
            }
            _ => {}
        }
    }

    /// Navigate to previous tab
    pub fn prev_tab(&mut self) {
        match self.view {
            View::EntityDetail => {
                self.entity_tab = match self.entity_tab {
                    EntityTab::Attributes => EntityTab::Metadata,
                    EntityTab::Relationships => EntityTab::Attributes,
                    EntityTab::Metadata => EntityTab::Relationships,
                };
            }
            View::UserDetail => {
                self.user_tab = match self.user_tab {
                    UserTab::DirectRoles => UserTab::Info,
                    UserTab::Teams => UserTab::DirectRoles,
                    UserTab::AllRoles => UserTab::Teams,
                    UserTab::Info => UserTab::AllRoles,
                };
                self.user_role_index = 0;
                self.user_team_index = 0;
            }
            _ => {}
        }
    }

    /// Get currently selected entity
    pub fn get_selected_entity(&self) -> Option<&EntityMetadata> {
        self.filtered_entities
            .get(self.entity_index)
            .and_then(|&i| self.entities.get(i))
    }

    /// Get currently selected solution
    pub fn get_selected_solution(&self) -> Option<&Solution> {
        self.filtered_solutions
            .get(self.solution_index)
            .and_then(|&i| self.solutions.get(i))
    }

    /// Get currently selected user
    pub fn get_selected_user(&self) -> Option<&SystemUser> {
        self.filtered_users
            .get(self.user_index)
            .and_then(|&i| self.users.get(i))
    }

    /// Enter detail view for selected entity
    pub fn enter_entity_detail(&mut self) {
        if let Some(entity) = self.get_selected_entity().cloned() {
            self.selected_entity = Some(entity);
            self.view = View::EntityDetail;
            self.search_query.clear();
        }
    }

    /// Enter detail view for selected user
    pub fn enter_user_detail(&mut self) {
        if let Some(user) = self.get_selected_user().cloned() {
            self.selected_user = Some(user);
            self.view = View::UserDetail;
            self.search_query.clear();
        }
    }

    /// Go back from detail view
    pub fn go_back(&mut self) {
        match self.view {
            View::EntityDetail => {
                self.view = View::Entities;
                self.search_query.clear();
            }
            View::SolutionDetail => {
                self.view = View::Solutions;
                self.search_query.clear();
            }
            View::UserDetail => {
                self.view = View::Users;
                self.search_query.clear();
            }
            _ => {}
        }
    }

    /// Execute the current query
    pub async fn execute_query(&mut self) {
        if self.query.entity_set_name.is_empty() {
            self.query_result.error = Some("Please select an entity first".to_string());
            return;
        }

        self.state = AppState::Loading;
        self.error = None;

        let url = self.query.build_url();

        match self.client.execute_query(&url).await {
            Ok(json) => {
                self.query_result = QueryResult::from_json(&json);
                self.query_result.raw_json = Some(serde_json::to_string_pretty(&json).unwrap_or_default());
                self.query_result_index = 0;
                self.state = AppState::Ready;
            }
            Err(e) => {
                self.query_result.error = Some(format!("Query failed: {}", e));
                self.state = AppState::Ready;
            }
        }
    }

    /// Set the entity for the query from current selection
    pub fn set_query_entity_from_selection(&mut self) {
        if let Some(entity) = self.get_selected_entity().cloned() {
            self.query.entity_name = entity.logical_name.clone();
            self.query.entity_set_name = entity.entity_set_name.clone().unwrap_or_else(|| {
                // Fallback: add 's' to logical name (not always correct but a guess)
                format!("{}s", entity.logical_name)
            });
            self.query_input = entity.logical_name;
        }
    }

    /// Apply the current query input to the appropriate field
    pub fn apply_query_input(&mut self) {
        match self.query_field {
            QueryField::Entity => {
                // Find entity by name
                if let Some(entity) = self.entities.iter().find(|e| {
                    e.logical_name.to_lowercase() == self.query_input.to_lowercase()
                }) {
                    self.query.entity_name = entity.logical_name.clone();
                    self.query.entity_set_name = entity.entity_set_name.clone().unwrap_or_else(|| {
                        format!("{}s", entity.logical_name)
                    });
                }
            }
            QueryField::Select => {
                self.query.select = self.query_input
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }
            QueryField::Filter => {
                self.query.filter = self.query_input.clone();
            }
            QueryField::OrderBy => {
                self.query.order_by = self.query_input.clone();
            }
            QueryField::Top => {
                self.query.top = self.query_input.parse().ok();
            }
        }
    }

    /// Load the current field value into input for editing
    pub fn load_query_field_to_input(&mut self) {
        self.query_input = match self.query_field {
            QueryField::Entity => self.query.entity_name.clone(),
            QueryField::Select => self.query.select.join(", "),
            QueryField::Filter => self.query.filter.clone(),
            QueryField::OrderBy => self.query.order_by.clone(),
            QueryField::Top => self.query.top.map(|n| n.to_string()).unwrap_or_default(),
        };
    }

    /// Clear the query and results
    pub fn clear_query(&mut self) {
        self.query.clear();
        self.query_input.clear();
        self.query_result = QueryResult::default();
        self.query_result_index = 0;
    }

    /// Navigate query results up
    pub fn query_result_up(&mut self) {
        if self.query_result_index > 0 {
            self.query_result_index -= 1;
        }
    }

    /// Navigate query results down
    pub fn query_result_down(&mut self) {
        if !self.query_result.rows.is_empty()
            && self.query_result_index < self.query_result.rows.len() - 1
        {
            self.query_result_index += 1;
        }
    }
}

