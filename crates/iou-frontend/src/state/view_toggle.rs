//! View toggle state for 2D/3D switching

use serde::{Deserialize, Serialize};

/// View mode for building layer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViewMode {
    /// 2D footprint view (flat polygons)
    TwoD,
    /// 3D extrusion view (height-based buildings)
    ThreeD,
}

impl ViewMode {
    /// Toggle to the other mode
    pub fn toggle(&mut self) {
        *self = match self {
            ViewMode::TwoD => ViewMode::ThreeD,
            ViewMode::ThreeD => ViewMode::TwoD,
        };
    }

    /// Returns true if 3D mode
    pub fn is_3d(&self) -> bool {
        matches!(self, ViewMode::ThreeD)
    }

    /// Returns the label for display
    pub fn label(&self) -> &'static str {
        match self {
            ViewMode::TwoD => "2D",
            ViewMode::ThreeD => "3D",
        }
    }

    /// Returns the button label (what happens when clicked)
    pub fn button_label(&self) -> &'static str {
        match self {
            ViewMode::TwoD => "3D Weergave",
            ViewMode::ThreeD => "2D Weergave",
        }
    }
}

impl Default for ViewMode {
    fn default() -> Self {
        ViewMode::ThreeD // Default to 3D (matches initial map state)
    }
}

/// View toggle state container
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ViewToggle {
    /// Current view mode
    pub mode: ViewMode,
}

impl ViewToggle {
    /// Creates a new ViewToggle with default mode (3D)
    pub fn new() -> Self {
        Self {
            mode: ViewMode::default(),
        }
    }

    /// Creates a ViewToggle with a specific mode
    pub fn with_mode(mode: ViewMode) -> Self {
        Self { mode }
    }

    /// Toggles the view mode
    pub fn toggle(&mut self) {
        self.mode.toggle();
    }

    /// Returns true if currently in 3D mode
    pub fn is_3d(&self) -> bool {
        self.mode.is_3d()
    }

    /// Returns the current mode label
    pub fn label(&self) -> &'static str {
        self.mode.label()
    }

    /// Returns the button label (what happens when clicked)
    pub fn button_label(&self) -> &'static str {
        self.mode.button_label()
    }
}

impl Default for ViewToggle {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_view_mode_default_is_3d() {
        let mode = ViewMode::default();
        assert_eq!(mode, ViewMode::ThreeD);
    }

    #[test]
    fn test_view_mode_toggle_switches_modes() {
        let mut mode = ViewMode::ThreeD;
        mode.toggle();
        assert_eq!(mode, ViewMode::TwoD);
        mode.toggle();
        assert_eq!(mode, ViewMode::ThreeD);
    }

    #[test]
    fn test_view_mode_is_3d() {
        assert!(ViewMode::ThreeD.is_3d());
        assert!(!ViewMode::TwoD.is_3d());
    }

    #[test]
    fn test_view_mode_label() {
        assert_eq!(ViewMode::TwoD.label(), "2D");
        assert_eq!(ViewMode::ThreeD.label(), "3D");
    }

    #[test]
    fn test_view_mode_button_label() {
        assert_eq!(ViewMode::TwoD.button_label(), "3D Weergave");
        assert_eq!(ViewMode::ThreeD.button_label(), "2D Weergave");
    }

    #[test]
    fn test_view_mode_serialization() {
        let mode = ViewMode::ThreeD;
        let json = serde_json::to_string(&mode);
        assert!(json.is_ok());
        let deserialized: ViewMode = serde_json::from_str(&json.unwrap()).unwrap();
        assert_eq!(mode, deserialized);
    }
}
