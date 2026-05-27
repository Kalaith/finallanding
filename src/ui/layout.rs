//! Main layout manager for UI regions

use macroquad::prelude::*;

/// Screen layout regions
pub struct Layout {
    pub top_bar_height: f32,
    pub left_panel_width: f32,
    pub right_panel_width: f32,
    pub bottom_toolbar_height: f32,
    pub screen_margin: f32,
}

impl Default for Layout {
    fn default() -> Self {
        Self {
            top_bar_height: 66.0,
            left_panel_width: 300.0,
            right_panel_width: 302.0,
            bottom_toolbar_height: 86.0,
            screen_margin: 12.0,
        }
    }
}

impl Layout {
    /// Get the game area rectangle (where grid is drawn)
    pub fn game_area(&self) -> Rect {
        Rect {
            x: self.left_panel_width,
            y: self.top_bar_height,
            w: (screen_width() - self.left_panel_width - self.right_panel_width).max(1.0),
            h: (screen_height() - self.top_bar_height - self.bottom_toolbar_height).max(1.0),
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
        self.right_panel()
    }

    pub fn left_panel(&self) -> Rect {
        Rect {
            x: self.screen_margin,
            y: self.top_bar_height + self.screen_margin,
            w: self.left_panel_width - self.screen_margin * 2.0,
            h: screen_height()
                - self.top_bar_height
                - self.bottom_toolbar_height
                - self.screen_margin * 2.0,
        }
    }

    pub fn right_panel(&self) -> Rect {
        Rect {
            x: screen_width() - self.right_panel_width + self.screen_margin,
            y: self.top_bar_height + self.screen_margin,
            w: self.right_panel_width - self.screen_margin * 2.0,
            h: screen_height()
                - self.top_bar_height
                - self.bottom_toolbar_height
                - self.screen_margin * 2.0,
        }
    }

    pub fn bottom_toolbar(&self) -> Rect {
        let width = (screen_width() * 0.46).clamp(520.0, 760.0);
        Rect {
            x: (screen_width() - width) * 0.5,
            y: screen_height() - self.bottom_toolbar_height + 10.0,
            w: width,
            h: self.bottom_toolbar_height - 20.0,
        }
    }
}
