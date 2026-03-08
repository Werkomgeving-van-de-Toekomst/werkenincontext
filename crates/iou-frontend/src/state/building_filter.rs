//! Building filter state for 3D map filtering

use serde::{Deserialize, Serialize};

/// Filter state for 3D building data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BuildingFilter {
    /// Minimum construction year (inclusive)
    pub min_year: u32,
    /// Maximum construction year (inclusive)
    pub max_year: u32,
    /// Minimum building height in meters (inclusive)
    pub min_height: f64,
    /// Maximum building height in meters (inclusive)
    pub max_height: f64,
    /// Minimum floor count (inclusive)
    pub min_floors: u32,
    /// Maximum floor count (inclusive)
    pub max_floors: u32,
}

impl BuildingFilter {
    /// Creates default filter state
    pub fn new() -> Self {
        Self {
            min_year: 1900,
            max_year: 2024,
            min_height: 0.0,
            max_height: 100.0,
            min_floors: 1,
            max_floors: 10,
        }
    }

    /// Returns true if no filters are active (all ranges at defaults)
    pub fn is_active(&self) -> bool {
        self.min_year != 1900 || self.max_year != 2024
            || self.min_height != 0.0 || self.max_height != 100.0
            || self.min_floors != 1 || self.max_floors != 10
    }

    /// Resets all filters to default values
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Validates that min <= max for all ranges
    pub fn is_valid(&self) -> bool {
        self.min_year <= self.max_year
            && self.min_height <= self.max_height
            && self.min_floors <= self.max_floors
    }
}

impl Default for BuildingFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_building_filter_new_creates_sensible_defaults() {
        let filter = BuildingFilter::new();
        assert_eq!(filter.min_year, 1900);
        assert_eq!(filter.max_year, 2024);
        assert_eq!(filter.min_height, 0.0);
        assert_eq!(filter.max_height, 100.0);
        assert_eq!(filter.min_floors, 1);
        assert_eq!(filter.max_floors, 10);
    }

    #[test]
    fn test_building_filter_is_active_false_for_defaults() {
        let filter = BuildingFilter::new();
        assert!(!filter.is_active());
    }

    #[test]
    fn test_building_filter_is_active_true_for_modified_year() {
        let mut filter = BuildingFilter::new();
        filter.min_year = 1950;
        assert!(filter.is_active());
    }

    #[test]
    fn test_building_filter_is_active_true_for_modified_height() {
        let mut filter = BuildingFilter::new();
        filter.max_height = 50.0;
        assert!(filter.is_active());
    }

    #[test]
    fn test_building_filter_is_active_true_for_modified_floors() {
        let mut filter = BuildingFilter::new();
        filter.min_floors = 3;
        assert!(filter.is_active());
    }

    #[test]
    fn test_building_filter_reset_clears_all_filters() {
        let mut filter = BuildingFilter::new();
        filter.min_year = 1950;
        filter.max_height = 50.0;
        filter.min_floors = 3;
        filter.reset();
        assert_eq!(filter, BuildingFilter::new());
    }

    #[test]
    fn test_building_filter_is_valid_true_for_sensible_ranges() {
        let filter = BuildingFilter::new();
        assert!(filter.is_valid());
    }

    #[test]
    fn test_building_filter_is_valid_false_for_inverted_year() {
        let mut filter = BuildingFilter::new();
        filter.min_year = 2000;
        filter.max_year = 1950;
        assert!(!filter.is_valid());
    }

    #[test]
    fn test_building_filter_is_valid_false_for_inverted_height() {
        let mut filter = BuildingFilter::new();
        filter.min_height = 50.0;
        filter.max_height = 10.0;
        assert!(!filter.is_valid());
    }

    #[test]
    fn test_building_filter_is_valid_false_for_inverted_floors() {
        let mut filter = BuildingFilter::new();
        filter.min_floors = 10;
        filter.max_floors = 3;
        assert!(!filter.is_valid());
    }

    #[test]
    fn test_building_filter_serialization() {
        let filter = BuildingFilter::new();
        let json = serde_json::to_string(&filter);
        assert!(json.is_ok());
        let deserialized: BuildingFilter = serde_json::from_str(&json.unwrap()).unwrap();
        assert_eq!(filter, deserialized);
    }
}
