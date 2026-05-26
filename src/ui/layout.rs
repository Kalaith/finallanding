//! Main layout manager for UI regions

use macroquad::prelude::*;

/// Screen layout regions
pub struct Layout {
    pub top_bar_height: f32,
    pub side_panel_width: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            top_bar_height: 50.0,
            side_panel_width: 220.0,
        }
    }
}

impl Layout {
    /// Get the game area rectangle (where grid is drawn)
    pub fn game_area(&self) -> Rect {
        Rect {
            x: 0.0,
            y: self.top_bar_height,
            w: screen_width() - self.side_panel_width,
            h: screen_height() - self.top_bar_height,
        }
    }

    /// Get the top bar rectangle
    pub fn top_bar(&self) -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            w: screen_width(),
            h: self.top_bar_height,
        }
    }

    /// Get the side panel rectangle
    pub fn side_panel(&self) -> Rect {
        Rect {
            x: screen_width() - self.side_panel_width,
            y: self.top_bar_height,
            w: self.side_panel_width,
            h: screen_height() - self.top_bar_height,
        }
    }
}
