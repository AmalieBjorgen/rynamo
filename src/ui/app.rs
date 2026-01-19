//! Application state and main TUI logic

use crate::api::DataverseClient;
use crate::models::{AttributeMetadata, EntityMetadata, RelationshipMetadata, Solution};
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
    /// All entities loaded
    pub entities: Vec<EntityMetadata>,
    /// Filtered entities (matching search)
    pub filtered_entities: Vec<usize>,
    /// Selected entity index in filtered list
    pub entity_index: usize,

    // Entity detail state
    /// Currently selected entity for detail view
    pub selected_entity: Option<EntityMetadata>,
    /// Attributes of selected entity
    pub entity_attributes: Vec<AttributeMetadata>,
    /// Filtered attributes
    pub filtered_attributes: Vec<usize>,
    /// Selected attribute index
    pub attribute_index: usize,
    /// One-to-many relationships
    pub one_to_many: Vec<RelationshipMetadata>,
    /// Many-to-one relationships
    pub many_to_one: Vec<RelationshipMetadata>,
    /// Many-to-many relationships
    pub many_to_many: Vec<RelationshipMetadata>,
    /// Current tab in entity detail
    pub entity_tab: EntityTab,
    /// Selected relationship index
    pub relationship_index: usize,

    // Solution list state
    /// All solutions
    pub solutions: Vec<Solution>,
    /// Filtered solutions
    pub filtered_solutions: Vec<usize>,
    /// Selected solution index
    pub solution_index: usize,

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
            should_quit: false,
        }
    }

    /// Load initial data (entities)
    pub async fn load_entities(&mut self) {
        self.state = AppState::Loading;
        self.error = None;

        match self.client.get_entities().await {
            Ok(mut entities) => {
                // Sort by logical name
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

        // Load attributes
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

        // Load relationships
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
            View::SolutionDetail => {}
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
            View::SolutionDetail => {}
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
            _ => {
                // Switch between main views
                self.view = match self.view {
                    View::Entities => View::Solutions,
                    View::Solutions => View::Entities,
                    other => other,
                };
            }
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
            _ => {
                self.view = match self.view {
                    View::Entities => View::Solutions,
                    View::Solutions => View::Entities,
                    other => other,
                };
            }
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

    /// Enter detail view for selected entity
    pub fn enter_entity_detail(&mut self) {
        if let Some(entity) = self.get_selected_entity().cloned() {
            self.selected_entity = Some(entity);
            self.view = View::EntityDetail;
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
            _ => {}
        }
    }
}
