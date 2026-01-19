//! Input handling and key bindings

use crossterm::event::KeyCode;

/// Whether vim-style keybindings are enabled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyBindings {
    /// Arrow keys for navigation (default)
    Arrows,
    /// Vim-style j/k navigation
    Vim,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::Arrows
    }
}

/// Current input mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    /// Normal navigation mode
    #[default]
    Normal,
    /// Search/filter input mode
    Search,
}

impl KeyBindings {
    /// Check if this key code moves up
    pub fn is_up(&self, key: KeyCode) -> bool {
        match (self, key) {
            (_, KeyCode::Up) => true,
            (Self::Vim, KeyCode::Char('k')) => true,
            _ => false,
        }
    }

    /// Check if this key code moves down
    pub fn is_down(&self, key: KeyCode) -> bool {
        match (self, key) {
            (_, KeyCode::Down) => true,
            (Self::Vim, KeyCode::Char('j')) => true,
            _ => false,
        }
    }

    /// Check if this key code moves left (for tabs)
    pub fn is_left(&self, key: KeyCode) -> bool {
        match (self, key) {
            (_, KeyCode::Left) => true,
            (Self::Vim, KeyCode::Char('h')) => true,
            _ => false,
        }
    }

    /// Check if this key code moves right (for tabs)
    pub fn is_right(&self, key: KeyCode) -> bool {
        match (self, key) {
            (_, KeyCode::Right) => true,
            (Self::Vim, KeyCode::Char('l')) => true,
            _ => false,
        }
    }
}
