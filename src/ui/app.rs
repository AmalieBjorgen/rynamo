//! Application state and main TUI logic

use crate::api::DataverseClient;
use crate::models::{
    AttributeMetadata, EntityMetadata, QueryResult,
    RelationshipMetadata, RoleAssignment, RoleSource, SecurityRole, Solution, SolutionComponent,
    ComponentType, SystemUser, Team, OptionSetMetadata, SystemJob,
};
use super::input::{InputMode, KeyBindings};
use anyhow::Context;
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
    RecordDetail,
    OptionSets,
    GlobalSearch,
    Environments,
    SolutionLayers,
    FetchXML,
    SystemJobs,
    SystemJobDetail,
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
    Query,
}

/// Query builder mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QueryMode {
    #[default]
    Columns,    // Select columns to include
    Filter,     // Build filter conditions
    OrderBy,    // Select order by column
    Options,    // Top, skip options
    Results,    // View results
}

#[derive(Debug, Clone)]
pub enum SearchResult {
    Entity(usize),     // Index in entities
    Solution(usize),   // Index in solutions
    OptionSet(usize),  // Index in global_optionsets
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

    // Solution detail state
    pub selected_solution: Option<Solution>,
    pub solution_components: Vec<SolutionComponent>,
    pub filtered_components: Vec<usize>,
    pub component_index: usize,

    // OptionSet state
    pub global_optionsets: Vec<OptionSetMetadata>,
    pub filtered_optionsets: Vec<usize>,
    pub optionset_index: usize,
    pub selected_optionset: Option<OptionSetMetadata>,

    // Global Search state
    pub global_search_results: Vec<SearchResult>,
    pub global_search_index: usize,

    // Environment state
    pub config: crate::config::Config,
    pub environment_index: usize,

    // Solution layers state
    pub solution_layers: Vec<crate::models::SolutionComponentLayer>,
    pub solution_layers_index: usize,

    // FetchXML state
    pub fetchxml_query: String,
    pub fetchxml_cursor: usize,

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

    // System Jobs state
    pub system_jobs: Vec<SystemJob>,
    pub filtered_system_jobs: Vec<usize>,
    pub system_job_index: usize,
    pub selected_system_job: Option<SystemJob>,
    pub system_jobs_next_link: Option<String>,
    pub should_load_more_jobs: bool,

    // Guided Query builder state (integrated in Entity Detail)
    pub query_mode: QueryMode,
    pub query_selected_columns: Vec<bool>,  // Parallel to entity_attributes - which are selected
    pub query_column_index: usize,          // Cursor in column list
    pub query_order_by: Option<usize>,      // Index of attribute to order by (None = no order)
    pub query_order_desc: bool,             // Order descending
    pub query_top: Option<usize>,           // $top value
    pub query_filter_attr: Option<usize>,   // Attribute index for filter
    pub query_filter_op: FilterOp,          // Filter operator
    pub query_filter_value: String,         // Filter value input
    pub query_filters: Vec<FilterCondition>, // Applied filters
    pub query_filter_index: usize,          // Cursor in filter list
    pub query_result: QueryResult,
    pub query_result_index: usize,
    pub query_editing: bool,                // Editing filter value

    // Record detail state
    pub selected_record_index: Option<usize>,
    pub record_detail_index: usize,

    // Feedback message
    pub message: Option<String>,

    /// Should quit
    pub should_quit: bool,
}

/// Filter operator for guided filter building
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FilterOp {
    #[default]
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    IsNull,
    IsNotNull,
}

impl FilterOp {
    pub fn to_odata(&self, attr: &str, value: &str) -> String {
        match self {
            Self::Equals => format!("{} eq '{}'", attr, value),
            Self::NotEquals => format!("{} ne '{}'", attr, value),
            Self::Contains => format!("contains({}, '{}')", attr, value),
            Self::StartsWith => format!("startswith({}, '{}')", attr, value),
            Self::EndsWith => format!("endswith({}, '{}')", attr, value),
            Self::GreaterThan => format!("{} gt {}", attr, value),
            Self::LessThan => format!("{} lt {}", attr, value),
            Self::IsNull => format!("{} eq null", attr),
            Self::IsNotNull => format!("{} ne null", attr),
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Equals => "equals",
            Self::NotEquals => "not equals",
            Self::Contains => "contains",
            Self::StartsWith => "starts with",
            Self::EndsWith => "ends with",
            Self::GreaterThan => ">",
            Self::LessThan => "<",
            Self::IsNull => "is null",
            Self::IsNotNull => "is not null",
        }
    }

    pub fn next(&self) -> Self {
        match self {
            Self::Equals => Self::NotEquals,
            Self::NotEquals => Self::Contains,
            Self::Contains => Self::StartsWith,
            Self::StartsWith => Self::EndsWith,
            Self::EndsWith => Self::GreaterThan,
            Self::GreaterThan => Self::LessThan,
            Self::LessThan => Self::IsNull,
            Self::IsNull => Self::IsNotNull,
            Self::IsNotNull => Self::Equals,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Self::Equals => Self::IsNotNull,
            Self::NotEquals => Self::Equals,
            Self::Contains => Self::NotEquals,
            Self::StartsWith => Self::Contains,
            Self::EndsWith => Self::StartsWith,
            Self::GreaterThan => Self::EndsWith,
            Self::LessThan => Self::GreaterThan,
            Self::IsNull => Self::LessThan,
            Self::IsNotNull => Self::IsNull,
        }
    }

    pub fn needs_value(&self) -> bool {
        !matches!(self, Self::IsNull | Self::IsNotNull)
    }
}

/// A single filter condition
#[derive(Debug, Clone)]
pub struct FilterCondition {
    pub attribute_name: String,
    pub operator: FilterOp,
    pub value: String,
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
            selected_solution: None,
            solution_components: Vec::new(),
            filtered_components: Vec::new(),
            component_index: 0,
            global_optionsets: Vec::new(),
            filtered_optionsets: Vec::new(),
            optionset_index: 0,
            selected_optionset: None,
            global_search_results: Vec::new(),
            global_search_index: 0,
            config: crate::config::Config::default(),
            environment_index: 0,
            solution_layers: Vec::new(),
            solution_layers_index: 0,
            fetchxml_query: String::new(),
            fetchxml_cursor: 0,
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
            query_mode: QueryMode::Columns,
            query_selected_columns: Vec::new(),
            query_column_index: 0,
            query_order_by: None,
            query_order_desc: false,
            query_top: Some(50), // Default top 50
            query_filter_attr: None,
            query_filter_op: FilterOp::Equals,
            query_filter_value: String::new(),
            query_filters: Vec::new(),
            query_filter_index: 0,
            query_result: QueryResult::default(),
            query_result_index: 0,
            query_editing: false,
            selected_record_index: None,
            record_detail_index: 0,
            message: None,
            should_quit: false,
            
            system_jobs: Vec::new(),
            filtered_system_jobs: Vec::new(),
            system_job_index: 0,
            selected_system_job: None,
            system_jobs_next_link: None,
            should_load_more_jobs: false,
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
        
        // Reset query state for new entity
        self.query_selected_columns = vec![false; self.entity_attributes.len()];
        self.query_column_index = 0;
        self.query_order_by = None;
        self.query_order_desc = false;
        self.query_filters.clear();
        self.query_filter_index = 0;
        self.query_result = QueryResult::default();
        self.query_result_index = 0;
        self.query_mode = QueryMode::Columns;
        
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

    /// Load system jobs
    pub async fn load_system_jobs(&mut self) {
        self.state = AppState::Loading;
        self.error = None;

        match self.client.get_system_jobs(50).await { // Default to top 50
            Ok((jobs, next_link)) => {
                self.filtered_system_jobs = (0..jobs.len()).collect();
                self.system_jobs = jobs;
                self.system_jobs_next_link = next_link;
                self.state = AppState::Ready;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load system jobs: {}", e));
                self.state = AppState::Error;
            }
        }
    }
    
    /// Load more system jobs (next page)
    pub async fn load_more_system_jobs(&mut self) {
        let next_link = match &self.system_jobs_next_link {
            Some(link) => link.clone(),
            None => return,
        };
        
        // Don't set loading state to avoid flickering, just append
        match self.client.get_next_page_system_jobs(&next_link).await {
            Ok((mut new_jobs, new_next_link)) => {
                let start_index = self.system_jobs.len();
                self.system_jobs.append(&mut new_jobs);
                
                // Append to filtered list if no filter is active. 
                // If filter IS active, we should re-filter everything or just append if they match? 
                // For simplicity, re-filter.
                self.filter_system_jobs();
                
                self.system_jobs_next_link = new_next_link;
            }
            Err(e) => {
                 self.error = Some(format!("Failed to load more jobs: {}", e));
                 // Don't change state to Error, just show message?
                 // For now, simple error handling
                 self.message = Some(format!("Error loading more: {}", e));
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

    /// Apply search filter to system jobs
    pub fn filter_system_jobs(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.filtered_system_jobs = (0..self.system_jobs.len()).collect();
        } else {
            self.filtered_system_jobs = self
                .system_jobs
                .iter()
                .enumerate()
                .filter(|(_, job)| {
                    job.get_name().to_lowercase().contains(&query) 
                    || job.get_status_label().to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.system_job_index = 0;
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
                EntityTab::Query => {
                    match self.query_mode {
                        QueryMode::Columns => {
                            if self.query_column_index > 0 {
                                self.query_column_index -= 1;
                            }
                        }
                        QueryMode::Filter => {
                            if self.query_filter_index > 0 {
                                self.query_filter_index -= 1;
                            }
                        }
                        QueryMode::Results => {
                            if self.query_result_index > 0 {
                                self.query_result_index -= 1;
                            }
                        }
                        _ => {}
                    }
                }
            },
            View::Solutions => {
                if self.solution_index > 0 {
                    self.solution_index -= 1;
                }
            }
            View::SolutionDetail => {
                if self.component_index > 0 {
                    self.component_index -= 1;
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
            View::RecordDetail => {
                if self.record_detail_index > 0 {
                    self.record_detail_index -= 1;
                }
            }
            View::OptionSets => {
                if self.optionset_index > 0 {
                    self.optionset_index -= 1;
                }
            }
            View::GlobalSearch => {
                if self.global_search_index > 0 {
                    self.global_search_index -= 1;
                }
            }
            View::Environments => {
                if self.environment_index > 0 {
                    self.environment_index -= 1;
                }
            }
            View::FetchXML => {}
            View::SolutionLayers => {
                if self.solution_layers_index > 0 {
                    self.solution_layers_index -= 1;
                }
            }
            View::SystemJobs => {
                if self.system_job_index > 0 {
                    self.system_job_index -= 1;
                }
            }
            View::SystemJobDetail => {}
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
                EntityTab::Query => {
                    match self.query_mode {
                        QueryMode::Columns => {
                            if !self.filtered_attributes.is_empty()
                                && self.query_column_index < self.filtered_attributes.len() - 1
                            {
                                self.query_column_index += 1;
                            }
                        }
                        QueryMode::Filter => {
                            if !self.query_filters.is_empty()
                                && self.query_filter_index < self.query_filters.len() - 1
                            {
                                self.query_filter_index += 1;
                            }
                        }
                        QueryMode::Results => {
                            if !self.query_result.rows.is_empty()
                                && self.query_result_index < self.query_result.rows.len() - 1
                            {
                                self.query_result_index += 1;
                            }
                        }
                        _ => {}
                    }
                }
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
            View::SolutionDetail => {
                if !self.filtered_components.is_empty()
                    && self.component_index < self.filtered_components.len() - 1
                {
                    self.component_index += 1;
                }
            }
            View::SystemJobs => {
                if !self.filtered_system_jobs.is_empty()
                    && self.system_job_index < self.filtered_system_jobs.len() - 1
                {
                    self.system_job_index += 1;
                    
                    // Trigger load more if near end
                    if self.system_job_index >= self.filtered_system_jobs.len() - 5 
                       && self.system_jobs_next_link.is_some() 
                       && self.search_query.is_empty() { // Only load more if not filtering locally
                        self.should_load_more_jobs = true;
                    }
                }
            }
            View::SystemJobDetail => {}
            View::RecordDetail => {
                if !self.query_result.columns.is_empty()
                    && self.record_detail_index < self.query_result.columns.len() - 1
                {
                    self.record_detail_index += 1;
                }
            }
            View::OptionSets => {
                if !self.filtered_optionsets.is_empty()
                    && self.optionset_index < self.filtered_optionsets.len() - 1
                {
                    self.optionset_index += 1;
                }
            }
            View::GlobalSearch => {
                if !self.global_search_results.is_empty()
                    && self.global_search_index < self.global_search_results.len() - 1
                {
                    self.global_search_index += 1;
                }
            }
            View::Environments => {
                if !self.config.environments.is_empty()
                    && self.environment_index < self.config.environments.len() - 1
                {
                    self.environment_index += 1;
                }
            }
            View::FetchXML => {}
            View::SolutionLayers => {
                if !self.solution_layers.is_empty()
                    && self.solution_layers_index < self.solution_layers.len() - 1
                {
                    self.solution_layers_index += 1;
                }
            }
        }
    }

    /// Navigate to next tab
    pub fn next_tab(&mut self) {
        match self.view {
            View::EntityDetail => {
                self.entity_tab = match self.entity_tab {
                    EntityTab::Attributes => EntityTab::Relationships,
                    EntityTab::Relationships => EntityTab::Metadata,
                    EntityTab::Metadata => EntityTab::Query,
                    EntityTab::Query => EntityTab::Attributes,
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
                    EntityTab::Attributes => EntityTab::Query,
                    EntityTab::Relationships => EntityTab::Attributes,
                    EntityTab::Metadata => EntityTab::Relationships,
                    EntityTab::Query => EntityTab::Metadata,
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
        let idx = self.filtered_entities.get(self.entity_index)?;
        self.entities.get(*idx)
    }

    pub fn get_selected_attribute(&self) -> Option<&crate::models::AttributeMetadata> {
        let idx = self.filtered_attributes.get(self.attribute_index)?;
        self.entity_attributes.get(*idx)
    }

    pub fn get_selected_component(&self) -> Option<&crate::models::SolutionComponent> {
        self.solution_components.get(self.component_index)
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

    /// Enter detail view for selected solution
    pub fn enter_solution_detail(&mut self) {
        if let Some(solution) = self.get_selected_solution().cloned() {
            self.selected_solution = Some(solution);
            self.view = View::SolutionDetail;
            self.search_query.clear();
            self.filtered_components.clear();
        }
    }

    /// Enter detail view for selected record
    pub fn enter_record_detail(&mut self) {
        if self.query_mode == QueryMode::Results && !self.query_result.rows.is_empty() {
             self.selected_record_index = Some(self.query_result_index);
             self.view = View::RecordDetail;
             self.record_detail_index = 0;
        }
    }

    /// Navigate to a related record from the current record detail view
    pub async fn navigate_to_related_record(&mut self) {
        let Some(row_idx) = self.selected_record_index else { return; };
        let col_idx = self.record_detail_index;
        
        let lookup = if let Some(lookup) = self.query_result.lookups.get(&(row_idx, col_idx)) {
            lookup.clone()
        } else {
            return;
        };

        // 1. Find the target entity metadata
        let target_entity = self.entities.iter().find(|e| e.logical_name == lookup.logical_name).cloned();
        
        if let Some(entity) = target_entity {
            // 2. Load entity detail (attributes, etc)
            self.load_entity_detail(&entity.logical_name).await;
            self.selected_entity = Some(entity);
            
            // 3. Fetch the specific record
            let logical_name = self.selected_entity.as_ref().unwrap().logical_name.clone();
            let entity_set = self.selected_entity.as_ref().unwrap().entity_set_name.clone().unwrap_or_else(|| {
                format!("{}s", logical_name)
            });
            let url = format!("{}({})", entity_set, lookup.id);
            
            self.state = AppState::Loading;
            self.error = None;
            
            match self.client.execute_query(&url).await {
                Ok(json) => {
                    // Wrap single object in a result format
                    let mut wrapped = serde_json::Map::new();
                    wrapped.insert("value".to_string(), serde_json::Value::Array(vec![(json.clone())]));
                    let wrapped_json = serde_json::Value::Object(wrapped);
                    
                    self.query_result = QueryResult::from_json(&wrapped_json);
                    self.query_result.raw_json = Some(serde_json::to_string_pretty(&json).unwrap_or_default());
                    
                    self.selected_record_index = Some(0);
                    self.record_detail_index = 0;
                    self.view = View::RecordDetail;
                    self.state = AppState::Ready;
                }
                Err(e) => {
                    self.error = Some(format!("Failed to fetch related record: {}", e));
                    self.state = AppState::Ready;
                }
            }
        } else {
            self.error = Some(format!("Entity metadata not found for: {}", lookup.logical_name));
        }
    }

    /// Load solution details (components)
    pub async fn load_solution_detail(&mut self, solution_id: &str) {
        self.state = AppState::Loading;
        self.error = None;

        match self.client.get_solution_components(solution_id).await {
            Ok(components) => {
                self.solution_components = components;
                self.filter_solution_components();
                self.component_index = 0;
                self.state = AppState::Ready;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load solution components: {}", e));
                self.state = AppState::Error;
            }
        }
    }

    /// Get selected solution component
    pub fn get_selected_solution_component(&self) -> Option<&SolutionComponent> {
        self.filtered_components
            .get(self.component_index)
            .and_then(|&i| self.solution_components.get(i))
    }

    /// Jump to the selected component if possible
    pub async fn jump_to_component(&mut self) -> bool {
        let Some(comp) = self.get_selected_solution_component().cloned() else { return false; };
        match comp.get_component_type() {
            ComponentType::Entity => {
                let Some(object_id) = &comp.object_id else { return false; };
                // Find entity by MetadataId
                let entity_logical_name = self.entities.iter()
                    .find(|e| e.metadata_id.to_lowercase() == object_id.to_lowercase())
                    .map(|e| e.logical_name.clone());
                
                if let Some(logical_name) = entity_logical_name {
                    self.load_entity_detail(&logical_name).await;
                    self.selected_entity = self.entities.iter().find(|e| e.logical_name == logical_name).cloned();
                    self.view = View::EntityDetail;
                    self.entity_tab = EntityTab::Attributes;
                    return true;
                }
            }
            _ => {
                // Not supported yet
            }
        }
        false
    }

    /// Filter solution components
    pub fn filter_solution_components(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.filtered_components = (0..self.solution_components.len()).collect();
        } else {
            self.filtered_components = self
                .solution_components
                .iter()
                .enumerate()
                .filter(|(_, c)| {
                    let type_name = c.get_component_type().display_name().to_lowercase();
                    let object_id = c.object_id.as_deref().unwrap_or("").to_lowercase();
                    
                    type_name.contains(&query) || object_id.contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.component_index = 0;
    }

    /// Load global option sets
    pub async fn load_global_optionsets(&mut self) {
        self.state = AppState::Loading;
        self.error = None;

        match self.client.get_global_option_sets().await {
            Ok(optionsets) => {
                self.global_optionsets = optionsets;
                self.filter_optionsets();
                self.state = AppState::Ready;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load global option sets: {}", e));
                self.state = AppState::Error;
            }
        }
    }

    /// Filter global option sets
    pub fn filter_optionsets(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.filtered_optionsets = (0..self.global_optionsets.len()).collect();
        } else {
            self.filtered_optionsets = self
                .global_optionsets
                .iter()
                .enumerate()
                .filter(|(_, os)| {
                    os.name.to_lowercase().contains(&query)
                        || os.get_display_name().to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }
        self.optionset_index = 0;
    }

    /// Execute search across all cached metadata
    pub fn execute_global_search(&mut self) {
        let query = self.search_query.to_lowercase();
        if query.is_empty() {
            self.global_search_results.clear();
            return;
        }

        let mut results = Vec::new();

        // 1. Entities
        for (i, entity) in self.entities.iter().enumerate() {
            if entity.logical_name.to_lowercase().contains(&query)
                || entity.get_display_name().to_lowercase().contains(&query)
            {
                results.push(SearchResult::Entity(i));
            }
        }

        // 2. Solutions
        for (i, solution) in self.solutions.iter().enumerate() {
            if solution.unique_name.to_lowercase().contains(&query)
                || solution.get_display_name().to_lowercase().contains(&query)
            {
                results.push(SearchResult::Solution(i));
            }
        }

        // 3. OptionSets
        for (i, os) in self.global_optionsets.iter().enumerate() {
            if os.name.to_lowercase().contains(&query)
                || os.get_display_name().to_lowercase().contains(&query)
            {
                results.push(SearchResult::OptionSet(i));
            }
        }

        self.global_search_results = results;
        self.global_search_index = 0;
        self.view = View::GlobalSearch;
    }

    /// Enter the selected search result
    pub async fn enter_search_result(&mut self) {
        let Some(result) = self.global_search_results.get(self.global_search_index).cloned() else { return; };
        match result {
            SearchResult::Entity(idx) => {
                let logical_name = self.entities[idx].logical_name.clone();
                self.load_entity_detail(&logical_name).await;
                self.selected_entity = Some(self.entities[idx].clone());
                self.view = View::EntityDetail;
            }
            SearchResult::Solution(idx) => {
                let solution_id = self.solutions[idx].solution_id.clone();
                self.load_solution_detail(&solution_id).await;
                self.selected_solution = Some(self.solutions[idx].clone());
                self.view = View::SolutionDetail;
            }
            SearchResult::OptionSet(idx) => {
                self.optionset_index = idx;
                // We need to find the filtered index for the OptionSet view
                if let Some(filtered_idx) = self.filtered_optionsets.iter().position(|&i| i == idx) {
                    self.optionset_index = filtered_idx;
                } else {
                    // Reset filter if not found
                    self.search_query.clear();
                    self.filter_optionsets();
                    self.optionset_index = idx;
                }
                self.view = View::OptionSets;
            }
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
            View::RecordDetail => {
                self.view = View::EntityDetail;
                self.selected_record_index = None;
            }
            View::SolutionLayers => {
                // Return to whatever made sense before.
                // If we have a selected solution detail, go there.
                // If we have an entity selected, go to entity detail.
                if self.selected_solution.is_some() {
                    self.view = View::SolutionDetail;
                } else if self.selected_entity.is_some() {
                    self.view = View::EntityDetail;
                } else {
                    self.view = View::Entities;
                }
            }
            _ => {}
        }
    }

    /// Build and execute query from guided selections
    pub async fn execute_guided_query(&mut self) {
        let Some(entity) = &self.selected_entity else {
            self.query_result.error = Some("No entity selected".to_string());
            return;
        };

        let entity_set = entity.entity_set_name.clone().unwrap_or_else(|| {
            format!("{}s", entity.logical_name)
        });

        // Build $select from selected columns
        let select: Vec<String> = self.query_selected_columns
            .iter()
            .enumerate()
            .filter(|(_, selected)| **selected)
            .filter_map(|(i, _)| self.entity_attributes.get(i))
            .map(|attr| attr.logical_name.clone())
            .collect();

        // Build $filter from filter conditions
        let filter_parts: Vec<String> = self.query_filters
            .iter()
            .map(|f| f.operator.to_odata(&f.attribute_name, &f.value))
            .collect();

        // Build URL
        let mut parts: Vec<String> = Vec::new();

        if !select.is_empty() {
            parts.push(format!("$select={}", select.join(",")));
        }

        if !filter_parts.is_empty() {
            parts.push(format!("$filter={}", filter_parts.join(" and ")));
        }

        if let Some(order_idx) = self.query_order_by {
            if let Some(attr) = self.entity_attributes.get(order_idx) {
                let dir = if self.query_order_desc { " desc" } else { "" };
                parts.push(format!("$orderby={}{}", attr.logical_name, dir));
            }
        }

        if let Some(top) = self.query_top {
            parts.push(format!("$top={}", top));
        }

        let url = if parts.is_empty() {
            entity_set
        } else {
            format!("{}?{}", entity_set, parts.join("&"))
        };

        self.state = AppState::Loading;
        self.error = None;

        match self.client.execute_query(&url).await {
            Ok(json) => {
                self.query_result = QueryResult::from_json(&json);
                self.query_result.raw_json = Some(serde_json::to_string_pretty(&json).unwrap_or_default());
                self.query_result_index = 0;
                self.query_mode = QueryMode::Results;
                self.state = AppState::Ready;
            }
            Err(e) => {
                self.query_result.error = Some(format!("Query failed: {}", e));
                self.state = AppState::Ready;
            }
        }
    }

    /// Load next page of query results
    pub async fn load_next_page(&mut self) {
        let Some(next_link) = self.query_result.next_link.clone() else {
            return;
        };

        self.state = AppState::Loading;
        self.error = None;

        match self.client.execute_query(&next_link).await {
            Ok(json) => {
                let next_result = QueryResult::from_json(&json);
                
                // Append new rows
                self.query_result.rows.extend(next_result.rows);
                self.query_result.next_link = next_result.next_link;
                self.query_result.raw_json = Some(serde_json::to_string_pretty(&json).unwrap_or_default());
                
                self.state = AppState::Ready;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load next page: {}", e));
                self.state = AppState::Ready;
            }
        }
    }

    /// Toggle column selection at current index
    pub fn toggle_query_column(&mut self) {
        if let Some(selected) = self.query_selected_columns.get_mut(self.query_column_index) {
            *selected = !*selected;
        }
    }

    /// Select all columns
    pub fn select_all_columns(&mut self) {
        for s in &mut self.query_selected_columns {
            *s = true;
        }
    }

    /// Clear all column selections
    pub fn clear_column_selections(&mut self) {
        for s in &mut self.query_selected_columns {
            *s = false;
        }
    }

    /// Add current filter to the list
    pub fn add_filter(&mut self) {
        if let Some(attr_idx) = self.query_filter_attr {
            if let Some(attr) = self.entity_attributes.get(attr_idx) {
                let filter = FilterCondition {
                    attribute_name: attr.logical_name.clone(),
                    operator: self.query_filter_op,
                    value: self.query_filter_value.clone(),
                };
                self.query_filters.push(filter);
                self.query_filter_value.clear();
                self.query_filter_attr = None;
            }
        }
    }

    /// Remove filter at current index
    pub fn remove_filter(&mut self) {
        if !self.query_filters.is_empty() && self.query_filter_index < self.query_filters.len() {
            self.query_filters.remove(self.query_filter_index);
            if self.query_filter_index > 0 {
                self.query_filter_index -= 1;
            }
        }
    }

    /// Clear query and results
    pub fn clear_query(&mut self) {
        for s in &mut self.query_selected_columns {
            *s = false;
        }
        self.query_filters.clear();
        self.query_order_by = None;
        self.query_order_desc = false;
        self.query_top = Some(50);
        self.query_result = QueryResult::default();
        self.query_result_index = 0;
        self.query_mode = QueryMode::Columns;
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

    /// Export current query results
    pub fn export_query_results(&mut self) {
        if self.query_result.rows.is_empty() {
            self.message = Some("No results to export".to_string());
            return;
        }

        let entity_name = self.selected_entity.as_ref().map(|e| e.logical_name.clone()).unwrap_or_else(|| "export".to_string());
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let filename = format!("{}_{}.csv", entity_name, timestamp);
        let path_str = format!("exports/{}", filename);
        let path = std::path::Path::new(&path_str);

        match crate::export::export_results(&self.query_result, crate::export::ExportFormat::Csv, path) {
            Ok(p) => self.message = Some(format!("Exported to {}", p)),
            Err(e) => self.message = Some(format!("Export failed: {}", e)),
        }
    }

    /// Clear the feedback message
    pub fn clear_message(&mut self) {
        self.message = None;
    }

    /// Switch to a different environment
    pub async fn switch_environment(&mut self, url: &str) -> anyhow::Result<()> {
        self.state = AppState::Loading;
        self.error = None;
        self.message = Some(format!("Connecting to {}...", url));

        let authenticator = std::sync::Arc::new(
            crate::auth::AzureAuthenticator::new(url)
                .await
                .context("Failed to create Azure authenticator")?,
        );

        if let Err(e) = authenticator.test_connection().await {
            self.error = Some(format!("Connection failed: {}", e));
            self.state = AppState::Error;
            return Err(anyhow::anyhow!("Connection failed: {}", e));
        }

        self.client = std::sync::Arc::new(crate::api::DataverseClient::new(authenticator));
        
        // Update config
        self.config.current_env = Some(url.to_string());
        let _ = self.config.save();

        // Reload data
        self.load_entities().await;
        self.view = View::Entities;
        self.message = Some(format!("Switched to {}", url));
        self.state = AppState::Ready;
        
        Ok(())
    }

    /// Add a new environment
    pub async fn add_new_environment(&mut self, url: String) -> anyhow::Result<()> {
        self.config.add_environment(url.clone());
        let _ = self.config.save();
        self.switch_environment(&url).await
    }

    /// Discover and add all available environments
    pub async fn discover_environments(&mut self) -> anyhow::Result<()> {
        self.state = AppState::Loading;
        self.message = Some("Discovering environments...".to_string());
        
        match self.client.discover_environments().await {
            Ok(instances) => {
                let mut added_count = 0;
                for instance in instances {
                    let url = instance.url.trim_end_matches('/').to_string();
                    if !self.config.environments.contains(&url) {
                        self.config.environments.push(url);
                        added_count += 1;
                    }
                }
                
                let _ = self.config.save();
                self.message = Some(format!("Discovered {} new environments", added_count));
                self.state = AppState::Ready;
                Ok(())
            }
            Err(e) => {
                self.error = Some(format!("Discovery failed: {}", e));
                self.state = AppState::Ready;
                Err(e)
            }
        }
    }

    /// Load solution layers for the current component
    pub async fn load_solution_layers(&mut self, component_id: &str, component_type: i32) {
        self.state = AppState::Loading;
        self.error = None;

        match self.client.get_solution_layers(component_id, component_type).await {
            Ok(layers) => {
                self.solution_layers = layers;
                self.solution_layers_index = 0;
                self.view = View::SolutionLayers;
                self.state = AppState::Ready;
            }
            Err(e) => {
                self.error = Some(format!("Failed to load solution layers: {}", e));
                self.state = AppState::Ready;
            }
        }
    }

    /// Execute the FetchXML query currently in the input
    pub async fn execute_fetch_xml_query(&mut self) {
        if self.fetchxml_query.is_empty() {
            return;
        }

        self.state = AppState::Loading;
        self.error = None;

        // Extract entity name from FetchXML (simple heuristic)
        let entity_name = if let Some(start) = self.fetchxml_query.find("<entity name=\"") {
            let rest = &self.fetchxml_query[start + 14..];
            if let Some(end) = rest.find("\"") {
                Some(&rest[..end])
            } else {
                None
            }
        } else if let Some(start) = self.fetchxml_query.find("<entity name='") {
            let rest = &self.fetchxml_query[start + 14..];
            if let Some(end) = rest.find("'") {
                Some(&rest[..end])
            } else {
                None
            }
        } else {
            None
        };

        let Some(logical_name) = entity_name else {
            self.error = Some("Could not find <entity name='...'> in FetchXML".to_string());
            self.state = AppState::Ready;
            return;
        };

        // We need the entity set name. If we don't have it, we might need to fetch it or guess it.
        // For now, let's try to find it from our cached entities.
        let entity_set_name = self.entities.iter()
            .find(|e| e.logical_name == logical_name)
            .and_then(|e| e.entity_set_name.clone())
            .unwrap_or_else(|| format!("{}s", logical_name));

        match self.client.execute_fetch_xml(&entity_set_name, &self.fetchxml_query).await {
            Ok(result) => {
                self.query_result = result;
                self.view = View::EntityDetail;
                self.entity_tab = EntityTab::Query;
                self.query_mode = QueryMode::Results;
                self.state = AppState::Ready;
            }
            Err(e) => {
                self.error = Some(format!("FetchXML execution failed: {}", e));
                self.state = AppState::Ready;
            }
        }
    }

    /// Show usage statistics for the selected attribute
    pub async fn show_attribute_usage(&mut self) {
        let Some(entity) = &self.selected_entity else { return; };
        
        let (logical_name, _metadata_id) = if let Some(attr) = self.get_selected_attribute() {
            (attr.logical_name.clone(), attr.metadata_id.clone())
        } else {
            return;
        };
        
        let entity_set_name = entity.entity_set_name.clone().unwrap_or_else(|| {
            format!("{}s", entity.logical_name)
        });
        
        self.message = Some(format!("Calculating usage for {}...", logical_name));
        
        match self.client.get_attribute_count(&entity_set_name, &logical_name).await {
            Ok(count) => {
                self.message = Some(format!("Attribute '{}' is populated in {} records", logical_name, count));
            }
            Err(e) => {
                self.error = Some(format!("Failed to calculate usage: {}", e));
            }
        }
    }
}
