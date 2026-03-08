//! URL state serialization for query parameters
//!
//! Enables shareable URLs by serializing application state to/from
//! human-readable query parameters (not opaque base64).

use serde::{Deserialize, Serialize};
use crate::state::{BuildingFilter, ViewToggle, DensityHeatmap, ViewMode};

/// Complete application state for URL sharing
///
/// Combines view mode, building filter, and density heatmap state
/// into a single serializable struct for URL persistence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UrlState {
    /// Current view mode (2D or 3D)
    #[serde(default)]
    pub view_mode: ViewMode,
    /// Building filter state
    #[serde(default)]
    pub filter: BuildingFilter,
    /// Density heatmap enabled state
    #[serde(default)]
    pub heatmap: DensityHeatmap,
}

impl UrlState {
    /// Serialize to URL query parameters string
    ///
    /// Returns human-readable query params like:
    /// "view=3d&year_min=1900&year_max=2024&height_min=0&height_max=100&floors_min=1&floors_max=10&heatmap=false"
    pub fn to_url_params(&self) -> String {
        let view = match self.view_mode {
            ViewMode::TwoD => "2d",
            ViewMode::ThreeD => "3d",
        };

        format!(
            "view={}&year_min={}&year_max={}&height_min={}&height_max={}&floors_min={}&floors_max={}&heatmap={}",
            view,
            self.filter.min_year,
            self.filter.max_year,
            self.filter.min_height,
            self.filter.max_height,
            self.filter.min_floors,
            self.filter.max_floors,
            self.heatmap.enabled
        )
    }

    /// Deserialize from URL query parameters
    ///
    /// Parses query string like "view=3d&year_min=1900&year_max=2024..."
    /// Missing parameters use defaults.
    pub fn from_url_params(query_string: &str) -> Self {
        let mut state = Self::default();

        // Parse query parameters
        for pair in query_string.split('&') {
            let mut parts = pair.split('=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                match key {
                    "view" => {
                        state.view_mode = match value {
                            "2d" => ViewMode::TwoD,
                            "3d" | _ => ViewMode::ThreeD,
                        };
                    }
                    "year_min" => {
                        if let Ok(year) = value.parse::<u32>() {
                            state.filter.min_year = year;
                        }
                    }
                    "year_max" => {
                        if let Ok(year) = value.parse::<u32>() {
                            state.filter.max_year = year;
                        }
                    }
                    "height_min" => {
                        if let Ok(height) = value.parse::<f64>() {
                            state.filter.min_height = height;
                        }
                    }
                    "height_max" => {
                        if let Ok(height) = value.parse::<f64>() {
                            state.filter.max_height = height;
                        }
                    }
                    "floors_min" => {
                        if let Ok(floors) = value.parse::<u32>() {
                            state.filter.min_floors = floors;
                        }
                    }
                    "floors_max" => {
                        if let Ok(floors) = value.parse::<u32>() {
                            state.filter.max_floors = floors;
                        }
                    }
                    "heatmap" => {
                        state.heatmap.enabled = match value {
                            "true" | "1" => true,
                            "false" | "0" | _ => false,
                        };
                    }
                    _ => {} // Ignore unknown parameters
                }
            }
        }

        state
    }

    /// Check if state is default (no active filters/settings)
    pub fn is_default(&self) -> bool {
        self.view_mode == ViewMode::default()
            && !self.filter.is_active()
            && !self.heatmap.is_enabled()
    }
}

impl Default for UrlState {
    fn default() -> Self {
        Self {
            view_mode: ViewMode::default(),
            filter: BuildingFilter::default(),
            heatmap: DensityHeatmap::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_state_to_url_params_generates_readable_params() {
        let state = UrlState {
            view_mode: ViewMode::ThreeD,
            filter: BuildingFilter {
                min_year: 1900,
                max_year: 2024,
                min_height: 0.0,
                max_height: 100.0,
                min_floors: 1,
                max_floors: 10,
            },
            heatmap: DensityHeatmap { enabled: false },
        };

        let params = state.to_url_params();

        assert!(params.contains("view=3d"), "Should contain view mode");
        assert!(params.contains("year_min=1900"), "Should contain min year");
        assert!(params.contains("year_max=2024"), "Should contain max year");
        assert!(params.contains("height_min=0"), "Should contain min height");
        assert!(params.contains("height_max=100"), "Should contain max height");
        assert!(params.contains("floors_min=1"), "Should contain min floors");
        assert!(params.contains("floors_max=10"), "Should contain max floors");
        assert!(params.contains("heatmap=false"), "Should contain heatmap state");
    }

    #[test]
    fn test_url_state_from_url_params_parses_correctly() {
        let params = "view=2d&year_min=1950&year_max=2020&height_min=5&height_max=50&floors_min=2&floors_max=8&heatmap=true";

        let state = UrlState::from_url_params(params);

        assert_eq!(state.view_mode, ViewMode::TwoD, "Should parse 2D view mode");
        assert_eq!(state.filter.min_year, 1950, "Should parse min year");
        assert_eq!(state.filter.max_year, 2020, "Should parse max year");
        assert_eq!(state.filter.min_height, 5.0, "Should parse min height");
        assert_eq!(state.filter.max_height, 50.0, "Should parse max height");
        assert_eq!(state.filter.min_floors, 2, "Should parse min floors");
        assert_eq!(state.filter.max_floors, 8, "Should parse max floors");
        assert_eq!(state.heatmap.enabled, true, "Should parse heatmap enabled");
    }

    #[test]
    fn test_url_state_default_returns_sensible_defaults() {
        let state = UrlState::default();

        assert_eq!(state.view_mode, ViewMode::ThreeD, "Default should be 3D view");
        assert_eq!(state.filter.min_year, 1900, "Default min year should be 1900");
        assert_eq!(state.filter.max_year, 2024, "Default max year should be 2024");
        assert_eq!(state.filter.min_height, 0.0, "Default min height should be 0");
        assert_eq!(state.filter.max_height, 100.0, "Default max height should be 100");
        assert_eq!(state.filter.min_floors, 1, "Default min floors should be 1");
        assert_eq!(state.filter.max_floors, 10, "Default max floors should be 10");
        assert_eq!(state.heatmap.enabled, false, "Default heatmap should be disabled");
    }

    #[test]
    fn test_url_state_roundtrip_preserves_all_values() {
        let original = UrlState {
            view_mode: ViewMode::TwoD,
            filter: BuildingFilter {
                min_year: 1950,
                max_year: 2020,
                min_height: 5.0,
                max_height: 50.0,
                min_floors: 2,
                max_floors: 8,
            },
            heatmap: DensityHeatmap { enabled: true },
        };

        let params = original.to_url_params();
        let restored = UrlState::from_url_params(&params);

        assert_eq!(original.view_mode, restored.view_mode, "View mode should match");
        assert_eq!(original.filter.min_year, restored.filter.min_year, "Min year should match");
        assert_eq!(original.filter.max_year, restored.filter.max_year, "Max year should match");
        assert_eq!(original.filter.min_height, restored.filter.min_height, "Min height should match");
        assert_eq!(original.filter.max_height, restored.filter.max_height, "Max height should match");
        assert_eq!(original.filter.min_floors, restored.filter.min_floors, "Min floors should match");
        assert_eq!(original.filter.max_floors, restored.filter.max_floors, "Max floors should match");
        assert_eq!(original.heatmap.enabled, restored.heatmap.enabled, "Heatmap state should match");
    }

    #[test]
    fn test_url_state_from_url_params_handles_missing_params() {
        // Only provide view mode, rest should use defaults
        let params = "view=2d";
        let state = UrlState::from_url_params(params);

        assert_eq!(state.view_mode, ViewMode::TwoD, "Should parse provided view mode");
        assert_eq!(state.filter.min_year, 1900, "Should use default min year");
        assert_eq!(state.filter.max_year, 2024, "Should use default max year");
        assert_eq!(state.filter.min_height, 0.0, "Should use default min height");
        assert_eq!(state.filter.max_height, 100.0, "Should use default max height");
        assert_eq!(state.filter.min_floors, 1, "Should use default min floors");
        assert_eq!(state.filter.max_floors, 10, "Should use default max floors");
        assert_eq!(state.heatmap.enabled, false, "Should use default heatmap state");
    }

    #[test]
    fn test_url_state_to_url_params_2d_mode() {
        let state = UrlState {
            view_mode: ViewMode::TwoD,
            ..Default::default()
        };

        let params = state.to_url_params();
        assert!(params.contains("view=2d"), "Should serialize 2D mode correctly");
    }

    #[test]
    fn test_url_state_is_default_true_for_defaults() {
        let state = UrlState::default();
        assert!(state.is_default(), "Default state should report as default");
    }

    #[test]
    fn test_url_state_is_default_false_for_modified_filter() {
        let mut state = UrlState::default();
        state.filter.min_year = 1950;
        assert!(!state.is_default(), "Modified filter should not be default");
    }

    #[test]
    fn test_url_state_is_default_false_for_modified_view() {
        let mut state = UrlState::default();
        state.view_mode = ViewMode::TwoD;
        assert!(!state.is_default(), "Modified view mode should not be default");
    }

    #[test]
    fn test_url_state_is_default_false_for_modified_heatmap() {
        let mut state = UrlState::default();
        state.heatmap.enabled = true;
        assert!(!state.is_default(), "Modified heatmap should not be default");
    }
}
