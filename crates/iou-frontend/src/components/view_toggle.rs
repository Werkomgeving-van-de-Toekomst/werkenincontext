//! View Toggle Component - Switch between 2D and 3D building views
//!
//! Uses single-layer architecture with setPaintProperty to prevent state desync.

use dioxus::prelude::*;
use crate::state::ViewMode;
use crate::state::BuildingFilter;

/// LocalStorage key for persisting view mode
const STORAGE_KEY: &'static str = "viewMode";

/// Builds JavaScript to set the building layer view mode (2D or 3D)
///
/// Uses single-layer architecture via setPaintProperty to avoid state desync
/// that would occur with dual-layer (separate 2d/3d layers) approach.
pub fn build_set_view_mode_script(mode: ViewMode) -> String {
    // MapLibre fill-extrusion-height property:
    // - Number: constant height for all buildings (0 = flat 2D footprint)
    // - Expression: ['coalesce', ['get', 'height'], 10] = use actual height
    let height_value = if mode.is_3d() {
        r#"["coalesce", ["get", "height"], 10]"#
    } else {
        "0"
    };

    // Also adjust pitch for visual feedback in 2D mode
    let pitch = if mode.is_3d() { 60 } else { 0 };

    format!(
        r#"
        (function() {{
            const map = window['map_map'];
            if (!map) {{
                console.error('Map not found for view mode update');
                return;
            }}

            try {{
                // Set building layer height (single-layer architecture)
                const layer = map.getLayer('building-3d');
                if (layer) {{
                    map.setPaintProperty('building-3d', 'fill-extrusion-height', {});
                    console.log('View mode set to: {}', {});
                }}

                // Adjust pitch for better 2D/3D visualization
                map.easeTo({{ pitch: {}, duration: 500 }});

                // Persist to localStorage
                localStorage.setItem('{}', '{}');

                console.log('View mode updated:', '{}');
            }} catch (e) {{
                console.error('View mode update error:', e);
            }}
        }})();
    "#,
        height_value,
        if mode.is_3d() { "3D" } else { "2D" },
        if mode.is_3d() { "true" } else { "false" },
        pitch,
        STORAGE_KEY,
        if mode.is_3d() { "3d" } else { "2d" },
        if mode.is_3d() { "3d" } else { "2d" }
    )
}

/// Builds JavaScript to get initial view mode from localStorage
///
/// Returns JavaScript that sets a global variable with the stored mode
/// or defaults to '3d' if not found.
pub fn build_get_initial_view_script() -> String {
    format!(
        r#"
        (function() {{
            try {{
                const stored = localStorage.getItem('{}');
                if (stored) {{
                    window['initialViewMode'] = stored;
                    console.log('Restored view mode from localStorage:', stored);
                }} else {{
                    window['initialViewMode'] = '3d';
                    console.log('No stored view mode, using default: 3d');
                }}
            }} catch (e) {{
                console.error('Error reading view mode from localStorage:', e);
                window['initialViewMode'] = '3d';
            }}
            return window['initialViewMode'];
        }})();
    "#,
        STORAGE_KEY
    )
}

/// View Toggle Button Component
///
/// Renders a button that switches between 2D footprint and 3D extrusion views.
/// The toggle state is persisted to localStorage.
#[component]
pub fn ViewToggle() -> Element {
    let mut view_mode = use_signal(|| ViewMode::default());

    // Initialize from localStorage on mount
    use_effect(move || {
        let script = build_get_initial_view_script();
        // Note: This script returns the value, but we can't easily capture it in Rust
        // Instead, we'll use a MutationObserver or sync approach
        // For now, we set the initial mode to the stored value via a direct JS call
        let sync_script = format!(
            r#"
            (function() {{
                try {{
                    const stored = localStorage.getItem('{}');
                    if (stored === '2d') {{
                        // Set 2D mode
                        const map = window['map_map'];
                        if (map) {{
                            map.setPaintProperty('building-3d', 'fill-extrusion-height', 0);
                            map.setPitch(0);
                        }}
                    }}
                }} catch (e) {{
                    console.error('Error restoring view mode:', e);
                }}
            }})();
            "#,
            STORAGE_KEY
        );
        document::eval(&sync_script);
    });

    let current_mode = *view_mode.read();

    let is_3d = current_mode.is_3d();

    rsx! {
        div {
            class: "view-toggle-container",

            div {
                class: "view-toggle-content",

                button {
                    class: if is_3d { "btn-3d-control active" } else { "btn-3d-control" },
                    onclick: move |_| {
                        view_mode.write().toggle();
                        let new_mode = *view_mode.read();
                        let script = build_set_view_mode_script(new_mode);
                        document::eval(&script);

                        // Update URL with new view mode
                        // Read filter state from current UI signals and heatmap from localStorage
                        let url_update_script = format!(
                            r#"
                            (function() {{
                                try {{
                                    // Read heatmap state from localStorage
                                    const heatmapEnabled = localStorage.getItem('densityHeatmapEnabled') === 'true';

                                    // Build URL params with new view mode
                                    const params = new URLSearchParams();
                                    params.set('view', '{}');

                                    // Note: We can't read the current filter values directly here,
                                    // so we preserve any existing filter params from the current URL
                                    const currentParams = new URLSearchParams(window.location.search);
                                    if (currentParams.has('year_min')) params.set('year_min', currentParams.get('year_min'));
                                    if (currentParams.has('year_max')) params.set('year_max', currentParams.get('year_max'));
                                    if (currentParams.has('height_min')) params.set('height_min', currentParams.get('height_min'));
                                    if (currentParams.has('height_max')) params.set('height_max', currentParams.get('height_max'));
                                    if (currentParams.has('floors_min')) params.set('floors_min', currentParams.get('floors_min'));
                                    if (currentParams.has('floors_max')) params.set('floors_max', currentParams.get('floors_max'));

                                    params.set('heatmap', heatmapEnabled.toString());

                                    // Update URL
                                    const url = new URL(window.location.href);
                                    url.search = params.toString();
                                    window.history.replaceState({{state: 'urlStateUpdated'}}, '', url);

                                    console.log('URL updated with view mode:', url.toString());
                                }} catch (e) {{
                                    console.error('Failed to update URL:', e);
                                }}
                            }})();
                            "#,
                            if new_mode.is_3d() { "3d" } else { "2d" }
                        );
                        document::eval(&url_update_script);
                    },
                    "{current_mode.button_label()}"
                }

                // Current mode indicator
                div {
                    class: "view-toggle-label",
                    "{current_mode.label()}"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_set_view_mode_script_2d_uses_zero_height() {
        let script = build_set_view_mode_script(ViewMode::TwoD);
        assert!(script.contains("'fill-extrusion-height', 0"));
    }

    #[test]
    fn test_build_set_view_mode_script_3d_uses_height_expression() {
        let script = build_set_view_mode_script(ViewMode::ThreeD);
        assert!(script.contains(r#"["coalesce", ["get", "height"], 10]"#));
    }

    #[test]
    fn test_build_set_view_mode_script_updates_pitch() {
        let script_2d = build_set_view_mode_script(ViewMode::TwoD);
        let script_3d = build_set_view_mode_script(ViewMode::ThreeD);

        assert!(script_2d.contains("pitch: 0"));
        assert!(script_3d.contains("pitch: 60"));
    }

    #[test]
    fn test_build_set_view_mode_script_saves_to_local_storage() {
        let script = build_set_view_mode_script(ViewMode::TwoD);
        assert!(script.contains("localStorage.setItem('viewMode', '2d')"));
    }

    #[test]
    fn test_build_set_view_mode_script_3d_saves_correct_storage() {
        let script = build_set_view_mode_script(ViewMode::ThreeD);
        assert!(script.contains("localStorage.setItem('viewMode', '3d')"));
    }

    #[test]
    fn test_build_get_initial_view_script_reads_from_local_storage() {
        let script = build_get_initial_view_script();
        assert!(script.contains("localStorage.getItem('viewMode')"));
    }

    #[test]
    fn test_build_get_initial_view_script_defaults_to_3d() {
        let script = build_get_initial_view_script();
        assert!(script.contains("'3d'"));
    }

    #[test]
    fn test_build_set_view_mode_script_uses_easeto_for_smooth_transition() {
        let script = build_set_view_mode_script(ViewMode::ThreeD);
        assert!(script.contains("easeTo"));
        assert!(script.contains("duration: 500"));
    }
}
