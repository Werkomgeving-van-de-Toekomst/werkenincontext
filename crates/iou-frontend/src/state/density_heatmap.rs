//! Density Heatmap state
//!
//! Manages the enabled/disabled state of the density heatmap overlay.

use serde::{Deserialize, Serialize};

/// Density heatmap state container
///
/// Simple boolean state for heatmap toggle.
/// More complex than ViewMode enum since it's just on/off,
/// but follows same pattern for consistency.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DensityHeatmap {
    /// Whether the density heatmap is currently enabled
    pub enabled: bool,
}

impl DensityHeatmap {
    /// Creates a new DensityHeatmap with disabled state (default)
    pub fn new() -> Self {
        Self {
            enabled: false,
        }
    }

    /// Creates a DensityHeatmap with a specific enabled state
    pub fn with_enabled(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Toggles the heatmap enabled state
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Returns true if heatmap is currently enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets the enabled state directly
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Returns the label for display
    pub fn label(&self) -> &'static str {
        if self.enabled {
            "Aan"
        } else {
            "Uit"
        }
    }

    /// Returns the button label (what happens when clicked)
    pub fn button_label(&self) -> &'static str {
        if self.enabled {
            "Verberg"
        } else {
            "Toon"
        }
    }
}

impl Default for DensityHeatmap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_density_heatmap_default_creates_disabled_state() {
        let heatmap = DensityHeatmap::default();
        assert!(!heatmap.enabled, "Default heatmap should be disabled");
        assert!(!heatmap.is_enabled(), "is_enabled() should return false for default");
    }

    #[test]
    fn test_density_heatmap_new_creates_disabled_state() {
        let heatmap = DensityHeatmap::new();
        assert!(!heatmap.enabled, "new() should create disabled state");
    }

    #[test]
    fn test_density_heatmap_toggle_switches_enabled_disabled() {
        let mut heatmap = DensityHeatmap::new();

        // Initial state should be disabled
        assert!(!heatmap.is_enabled());

        // First toggle should enable
        heatmap.toggle();
        assert!(heatmap.is_enabled(), "After first toggle, should be enabled");

        // Second toggle should disable
        heatmap.toggle();
        assert!(!heatmap.is_enabled(), "After second toggle, should be disabled");
    }

    #[test]
    fn test_density_heatmap_is_enabled_returns_correct_boolean() {
        let disabled = DensityHeatmap::with_enabled(false);
        assert!(!disabled.is_enabled(), "with_enabled(false) should make is_enabled() return false");

        let enabled = DensityHeatmap::with_enabled(true);
        assert!(enabled.is_enabled(), "with_enabled(true) should make is_enabled() return true");
    }

    #[test]
    fn test_density_heatmap_set_enabled_updates_state() {
        let mut heatmap = DensityHeatmap::new();

        assert!(!heatmap.is_enabled());
        heatmap.set_enabled(true);
        assert!(heatmap.is_enabled());
        heatmap.set_enabled(false);
        assert!(!heatmap.is_enabled());
    }

    #[test]
    fn test_density_heatmap_with_enabled_creates_correct_state() {
        let enabled_heatmap = DensityHeatmap::with_enabled(true);
        assert!(enabled_heatmap.enabled, "with_enabled(true) should set enabled to true");

        let disabled_heatmap = DensityHeatmap::with_enabled(false);
        assert!(!disabled_heatmap.enabled, "with_enabled(false) should set enabled to false");
    }

    #[test]
    fn test_density_heatmap_serialization_preserves_state() {
        let original = DensityHeatmap::with_enabled(true);
        let json = serde_json::to_string(&original);
        assert!(json.is_ok(), "Serialization should succeed");

        let deserialized: DensityHeatmap = serde_json::from_str(&json.unwrap()).unwrap();
        assert_eq!(original.enabled, deserialized.enabled, "Enabled state should be preserved through serialization");
    }

    #[test]
    fn test_density_heatmap_label_returns_dutch_text() {
        let enabled = DensityHeatmap::with_enabled(true);
        assert_eq!(enabled.label(), "Aan", "Enabled heatmap should show 'Aan'");

        let disabled = DensityHeatmap::with_enabled(false);
        assert_eq!(disabled.label(), "Uit", "Disabled heatmap should show 'Uit'");
    }

    #[test]
    fn test_density_heatmap_button_label_shows_action() {
        let enabled = DensityHeatmap::with_enabled(true);
        assert_eq!(enabled.button_label(), "Verberg", "Enabled heatmap button should show 'Verberg'");

        let disabled = DensityHeatmap::with_enabled(false);
        assert_eq!(disabled.button_label(), "Toon", "Disabled heatmap button should show 'Toon'");
    }
}
