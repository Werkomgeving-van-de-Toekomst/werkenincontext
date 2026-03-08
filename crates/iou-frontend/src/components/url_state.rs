//! URL state manager component with JavaScript interop
//!
//! Provides functions to update browser URL via history.replaceState()
//! and restore state from window.location.search on page load.

use crate::state::UrlState;

/// Builds JavaScript to update URL query parameters from UrlState
///
/// Uses history.replaceState() to avoid creating browser history entries
/// for every state change. The URL is updated in-place.
///
/// # Arguments
/// * `state` - Current application state to serialize to URL
///
/// # Returns
/// JavaScript code that updates window.location.search with query params
pub fn build_update_url_script(state: &UrlState) -> String {
    let params = state.to_url_params();

    format!(
        r#"
        (function() {{
            try {{
                const params = '{params}';
                const url = new URL(window.location.href);
                url.search = params;

                // Update URL without adding history entry
                window.history.replaceState({{state: 'urlStateUpdated'}}, '', url);

                console.log('URL updated:', url.toString());
            }} catch (e) {{
                console.error('Failed to update URL:', e);
            }}
        }})();
        "#,
        params = params
    )
}

/// Builds JavaScript to restore state from URL query parameters
///
/// Reads window.location.search on page load and parses into UrlState.
/// Sets a global window['initialUrlState'] variable with the parsed state.
///
/// # Returns
/// JavaScript code that reads URL params and sets global variable
pub fn build_restore_state_script() -> String {
    r#"
    (function() {
        try {
            const params = new URLSearchParams(window.location.search);
            const queryString = params.toString() || '';

            // Set global variable for Rust to read
            window['initialUrlParams'] = queryString;
            console.log('Restored URL params:', queryString);
        } catch (e) {
            console.error('Failed to restore URL state:', e);
            window['initialUrlParams'] = '';
        }
    })();
    "#.to_string()
}

/// Builds JavaScript to get current URL parameters as a string
///
/// # Returns
/// JavaScript code that returns current query parameters string
pub fn build_get_url_params_script() -> String {
    r#"
    (function() {
        try {
            const params = new URLSearchParams(window.location.search);
            return params.toString() || '';
        } catch (e) {
            console.error('Failed to get URL params:', e);
            return '';
        }
    })();
    "#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{ViewMode, BuildingFilter, DensityHeatmap};

    #[test]
    fn test_build_update_url_script_generates_valid_javascript() {
        let state = UrlState::default();
        let script = build_update_url_script(&state);

        assert!(script.contains("window.history.replaceState"),
                "Script should call history.replaceState()");
        assert!(script.contains("new URL(window.location.href)"),
                "Script should create URL object");
        assert!(script.contains("url.search"),
                "Script should set search property");
    }

    #[test]
    fn test_build_update_url_script_includes_query_params() {
        let state = UrlState {
            view_mode: ViewMode::ThreeD,
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

        let script = build_update_url_script(&state);

        assert!(script.contains("view=3d"),
                "Script should include view mode param");
        assert!(script.contains("year_min=1950"),
                "Script should include min year param");
        assert!(script.contains("year_max=2020"),
                "Script should include max year param");
    }

    #[test]
    fn test_build_update_url_script_uses_replacestate_not_pushstate() {
        let state = UrlState::default();
        let script = build_update_url_script(&state);

        assert!(script.contains("replaceState"),
                "Script should use replaceState, not pushState");
        assert!(!script.contains("pushState"),
                "Script should NOT use pushState");
    }

    #[test]
    fn test_build_restore_state_script_reads_from_window_location() {
        let script = build_restore_state_script();

        assert!(script.contains("window.location.search"),
                "Script should read from window.location.search");
        assert!(script.contains("window['initialUrlParams']"),
                "Script should set global variable");
    }

    #[test]
    fn test_build_restore_state_script_handles_empty_query_string() {
        let script = build_restore_state_script();

        assert!(script.contains("params.toString() || ''"),
                "Script should handle empty params");
    }

    #[test]
    fn test_build_get_url_params_script_returns_string() {
        let script = build_get_url_params_script();

        assert!(script.contains("new URLSearchParams(window.location.search)"),
                "Script should read URL params");
        assert!(script.contains("return params.toString()"),
                "Script should return params string");
    }

    #[test]
    fn test_build_update_url_script_wraps_in_try_catch() {
        let state = UrlState::default();
        let script = build_update_url_script(&state);

        assert!(script.contains("try {"),
                "Script should have try block");
        assert!(script.contains("} catch (e)"),
                "Script should have catch block");
        assert!(script.contains("console.error"),
                "Script should log errors");
    }

    #[test]
    fn test_build_restore_state_script_wraps_in_try_catch() {
        let script = build_restore_state_script();

        assert!(script.contains("try {"),
                "Script should have try block");
        assert!(script.contains("} catch (e)"),
                "Script should have catch block");
    }

    #[test]
    fn test_build_update_url_script_logs_url_update() {
        let state = UrlState::default();
        let script = build_update_url_script(&state);

        assert!(script.contains("console.log('URL updated:'"),
                "Script should log URL update");
    }
}
