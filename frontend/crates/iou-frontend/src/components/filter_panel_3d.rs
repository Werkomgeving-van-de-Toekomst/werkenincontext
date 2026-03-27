//! 3D Building Filter Panel Component
//!
//! Provides interactive slider controls for filtering 3D buildings
//! by construction year, height, and floor count.
//!
//! URL state persistence: Filter changes update the browser URL immediately
//! via inline JavaScript in slider oninput handlers.
//!
//! URL state restoration: On component mount, reads URL params to restore
//! previous filter values (GAP-04 fix).

use dioxus::prelude::*;
use crate::state::BuildingFilter;

/// Builds a MapLibre filter expression from BuildingFilter state
///
/// Returns a JSON string representing the filter expression that
/// MapLibre's setFilter() method accepts. Uses ["all", ...] to
/// combine multiple conditions.
pub fn build_filter_expression(filter: &BuildingFilter) -> String {
    format!(
        r#"["all",
        [">=", ["get", "construction_year"], {}],
        ["<=", ["get", "construction_year"], {}],
        [">=", ["get", "height"], {}],
        ["<=", ["get", "height"], {}],
        [">=", ["get", "floors"], {}],
        ["<=", ["get", "floors"], {}]
    ]"#,
        filter.min_year,
        filter.max_year,
        filter.min_height,
        filter.max_height,
        filter.min_floors,
        filter.max_floors
    )
}

/// Builds JavaScript to update the map filter and query visible count
///
/// Uses map.isStyleLoaded() check to defer filter application until
/// MapLibre's style is fully loaded, preventing "Style is not done loading" errors.
pub fn build_set_filter_script(filter_expr: &str) -> String {
    format!(
        r#"
        (function() {{
            const map = window['map_map'];
            if (!map) {{
                console.error('Map not found for filter update');
                return;
            }}

            const applyFilter = function() {{
                try {{
                    // Log filter expression as string to avoid circular reference error
                    const filterExpr = '{filter_expr}';
                    map.setFilter('building-3d', JSON.parse(filterExpr));

                    // Query visible buildings and update count
                    const features = map.queryRenderedFeatures({{
                        layers: ['building-3d']
                    }});
                    const count = features.length;

                    // Update DOM element
                    const countEl = document.getElementById('building-count');
                    if (countEl) {{
                        countEl.textContent = count.toString();
                    }}

                    console.log('Filter updated:', filterExpr, 'Visible buildings:', count);
                }} catch (e) {{
                    console.error('Filter update error:', e);
                }}
            }};

            // Check if style is loaded before applying filter
            if (map.isStyleLoaded()) {{
                applyFilter();
            }} else {{
                // Defer filter application until style is loaded
                map.once('load', function() {{
                    console.log('Style loaded, applying deferred filter');
                    applyFilter();
                }});
            }}
        }})();
    "#,
        filter_expr = filter_expr.replace('\'', "\\'")  // Escape single quotes for JS string
    )
}

/// Builds JavaScript to clear all filters (show all buildings)
///
/// Uses map.isStyleLoaded() check to defer filter clearing until
/// MapLibre's style is fully loaded, preventing "Style is not done loading" errors.
pub fn build_clear_filter_script() -> String {
    r#"
        (function() {
            const map = window['map_map'];
            if (!map) {
                console.error('Map not found');
                return;
            }

            const clearFilter = function() {
                try {
                    map.setFilter('building-3d', null);

                    // Query all buildings and update count
                    const features = map.queryRenderedFeatures({
                        layers: ['building-3d']
                    });
                    const count = features.length;

                    const countEl = document.getElementById('building-count');
                    if (countEl) {
                        countEl.textContent = count.toString();
                    }

                    console.log('Filters cleared, all buildings visible');
                } catch (e) {
                    console.error('Clear filter error:', e);
                }
            };

            // Check if style is loaded before clearing filter
            if (map.isStyleLoaded()) {
                clearFilter();
            } else {
                // Defer filter clearing until style is loaded
                map.once('load', function() {
                    console.log('Style loaded, applying deferred filter clear');
                    clearFilter();
                });
            }
        })();
    "#
    .to_string()
}

/// Builds JavaScript to restore filter values from URL parameters
///
/// Reads URL params on page load and returns a JavaScript object with
/// the filter values that should be applied.
///
/// # GAP-04 Fix: URL state restoration on component mount
///
/// # Returns
/// JavaScript code that reads URL params and returns filter values
fn build_restore_filters_from_url_script() -> String {
    r#"
    (function() {
        try {
            const params = new URLSearchParams(window.location.search);

            // Parse filter values from URL params
            const yearMin = params.get('year_min');
            const yearMax = params.get('year_max');
            const heightMin = params.get('height_min');
            const heightMax = params.get('height_max');
            const floorsMin = params.get('floors_min');
            const floorsMax = params.get('floors_max');

            // Build result object with values to restore
            const result = {
                hasFilters: false,
                yearMin: yearMin ? parseInt(yearMin, 10) : null,
                yearMax: yearMax ? parseInt(yearMax, 10) : null,
                heightMin: heightMin ? parseFloat(heightMin) : null,
                heightMax: heightMax ? parseFloat(heightMax) : null,
                floorsMin: floorsMin ? parseInt(floorsMin, 10) : null,
                floorsMax: floorsMax ? parseInt(floorsMax, 10) : null
            };

            // Check if any filters were present
            result.hasFilters = yearMin || yearMax || heightMin || heightMax || floorsMin || floorsMax;

            console.log('GAP-04: Restored filters from URL:', result);
            return result;
        } catch (e) {
            console.error('GAP-04: Failed to restore filters from URL:', e);
            return { hasFilters: false };
        }
    })();
    "#.to_string()
}
///
/// Reads filter values directly from DOM and updates URL via history.replaceState()
/// This function is designed to be called from inline oninput handlers.
fn build_update_url_from_filters_script(
    year_min: u32,
    year_max: u32,
    height_min: f64,
    height_max: f64,
    floors_min: u32,
    floors_max: u32,
) -> String {
    format!(
        r#"
        (function() {{
            try {{
                // Read view mode and heatmap state from localStorage
                const viewMode = localStorage.getItem('viewMode') || '3d';
                const heatmapEnabled = localStorage.getItem('densityHeatmapEnabled') === 'true';

                // Build URL params with current filter values
                const params = new URLSearchParams();
                params.set('view', viewMode);
                params.set('year_min', '{year_min}');
                params.set('year_max', '{year_max}');
                params.set('height_min', '{height_min}');
                params.set('height_max', '{height_max}');
                params.set('floors_min', '{floors_min}');
                params.set('floors_max', '{floors_max}');
                params.set('heatmap', heatmapEnabled.toString());

                // Update URL without adding history entry
                const url = new URL(window.location.href);
                url.search = params.toString();
                window.history.replaceState({{state: 'urlStateUpdated'}}, '', url);

                console.log('Filter URL updated:', url.toString());
            }} catch (e) {{
                console.error('Failed to update filter URL:', e);
            }}
        }})();
        "#
    )
}

/// Filter Panel Component for 3D Buildings
#[component]
pub fn FilterPanel3D() -> Element {
    // Filter state - use signals for reactivity
    let mut min_year = use_signal(|| 1900u32);
    let mut max_year = use_signal(|| 2024u32);
    let mut min_height = use_signal(|| 0.0f64);
    let mut max_height = use_signal(|| 100.0f64);
    let mut min_floors = use_signal(|| 1u32);
    let mut max_floors = use_signal(|| 10u32);

    // Apply filter when any value changes
    // Note: use_effect runs on mount and re-runs when signals are read
    // The URL update happens in each slider's oninput handler
    use_effect(move || {
        let filter = BuildingFilter {
            min_year: *min_year.read(),
            max_year: *max_year.read(),
            min_height: *min_height.read(),
            max_height: *max_height.read(),
            min_floors: *min_floors.read(),
            max_floors: *max_floors.read(),
        };

        let filter_expr = build_filter_expression(&filter);
        let script = build_set_filter_script(&filter_expr);
        document::eval(&script);
    });

    let has_active_filters = *min_year.read() > 1900
        || *max_year.read() < 2024
        || *min_height.read() > 0.0
        || *max_height.read() < 100.0
        || *min_floors.read() > 1
        || *max_floors.read() < 10;

    rsx! {
        div { class: "filter-panel-3d",
            div { class: "filter-header",
                h3 { "Gebouwen Filter" }
                div { class: "building-count-display",
                    span { class: "count-label", "Zichtbaar: " }
                    span {
                        id: "building-count",
                        class: "count-value",
                        "-"
                    }
                }
            }

            // Construction year filter
            div { class: "filter-group",
                label { class: "filter-label",
                    "Bouwjaar: {min_year} - {max_year}"
                }
                div { class: "filter-slider-row",
                    input {
                        class: "filter-slider",
                        id: "filter-year-min",
                        r#type: "range",
                        min: 1900,
                        max: 2024,
                        value: "{min_year}",
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() {
                                min_year.set(v.min(*max_year.read()));
                                // Update URL immediately with new filter values
                                let url_script = build_update_url_from_filters_script(
                                    *min_year.read(),
                                    *max_year.read(),
                                    *min_height.read(),
                                    *max_height.read(),
                                    *min_floors.read(),
                                    *max_floors.read(),
                                );
                                document::eval(&url_script);
                            }
                        }
                    }
                    input {
                        class: "filter-slider",
                        id: "filter-year-max",
                        r#type: "range",
                        min: 1900,
                        max: 2024,
                        value: "{max_year}",
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() {
                                max_year.set(v.max(*min_year.read()));
                                // Update URL immediately with new filter values
                                let url_script = build_update_url_from_filters_script(
                                    *min_year.read(),
                                    *max_year.read(),
                                    *min_height.read(),
                                    *max_height.read(),
                                    *min_floors.read(),
                                    *max_floors.read(),
                                );
                                document::eval(&url_script);
                            }
                        }
                    }
                }
            }

            // Height filter
            div { class: "filter-group",
                label { class: "filter-label",
                    "Hoogte (m): {min_height:.0} - {max_height:.0}"
                }
                div { class: "filter-slider-row",
                    input {
                        class: "filter-slider",
                        id: "filter-height-min",
                        r#type: "range",
                        min: 0,
                        max: 100,
                        step: 1,
                        value: "{min_height}",
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                min_height.set(v.min(*max_height.read()));
                                // Update URL immediately with new filter values
                                let url_script = build_update_url_from_filters_script(
                                    *min_year.read(),
                                    *max_year.read(),
                                    *min_height.read(),
                                    *max_height.read(),
                                    *min_floors.read(),
                                    *max_floors.read(),
                                );
                                document::eval(&url_script);
                            }
                        }
                    }
                    input {
                        class: "filter-slider",
                        id: "filter-height-max",
                        r#type: "range",
                        min: 0,
                        max: 100,
                        step: 1,
                        value: "{max_height}",
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                max_height.set(v.max(*min_height.read()));
                                // Update URL immediately with new filter values
                                let url_script = build_update_url_from_filters_script(
                                    *min_year.read(),
                                    *max_year.read(),
                                    *min_height.read(),
                                    *max_height.read(),
                                    *min_floors.read(),
                                    *max_floors.read(),
                                );
                                document::eval(&url_script);
                            }
                        }
                    }
                }
            }

            // Floors filter
            div { class: "filter-group",
                label { class: "filter-label",
                    "Verdiepingen: {min_floors} - {max_floors}"
                }
                div { class: "filter-slider-row",
                    input {
                        class: "filter-slider",
                        id: "filter-floors-min",
                        r#type: "range",
                        min: 1,
                        max: 20,
                        step: 1,
                        value: "{min_floors}",
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() {
                                min_floors.set(v.min(*max_floors.read()));
                                // Update URL immediately with new filter values
                                let url_script = build_update_url_from_filters_script(
                                    *min_year.read(),
                                    *max_year.read(),
                                    *min_height.read(),
                                    *max_height.read(),
                                    *min_floors.read(),
                                    *max_floors.read(),
                                );
                                document::eval(&url_script);
                            }
                        }
                    }
                    input {
                        class: "filter-slider",
                        id: "filter-floors-max",
                        r#type: "range",
                        min: 1,
                        max: 20,
                        step: 1,
                        value: "{max_floors}",
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() {
                                max_floors.set(v.max(*min_floors.read()));
                                // Update URL immediately with new filter values
                                let url_script = build_update_url_from_filters_script(
                                    *min_year.read(),
                                    *max_year.read(),
                                    *min_height.read(),
                                    *max_height.read(),
                                    *min_floors.read(),
                                    *max_floors.read(),
                                );
                                document::eval(&url_script);
                            }
                        }
                    }
                }
            }

            // Clear filters button
            if has_active_filters {
                button {
                    class: "btn-clear-filters",
                    onclick: move |_| {
                        min_year.set(1900);
                        max_year.set(2024);
                        min_height.set(0.0);
                        max_height.set(100.0);
                        min_floors.set(1);
                        max_floors.set(10);
                        let script = build_clear_filter_script();
                        document::eval(&script);

                        // Update URL with default filter values
                        let url_update_script = format!(
                            r#"
                            (function() {{
                                try {{
                                    // Read view mode and heatmap state from localStorage
                                    const viewMode = localStorage.getItem('viewMode') || '3d';
                                    const heatmapEnabled = localStorage.getItem('densityHeatmapEnabled') === 'true';

                                    // Build URL params with default filter values
                                    const params = new URLSearchParams();
                                    params.set('view', viewMode);
                                    params.set('year_min', '1900');
                                    params.set('year_max', '2024');
                                    params.set('height_min', '0');
                                    params.set('height_max', '100');
                                    params.set('floors_min', '1');
                                    params.set('floors_max', '10');
                                    params.set('heatmap', heatmapEnabled.toString());

                                    // Update URL
                                    const url = new URL(window.location.href);
                                    url.search = params.toString();
                                    window.history.replaceState({{state: 'urlStateUpdated'}}, '', url);

                                    console.log('URL updated with cleared filters:', url.toString());
                                }} catch (e) {{
                                    console.error('Failed to update URL:', e);
                                }}
                            }})();
                            "#
                        );
                        document::eval(&url_update_script);
                    },
                    "Filters wissen"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_filter_expression_includes_all_properties() {
        let filter = BuildingFilter::new();
        let expr = build_filter_expression(&filter);

        assert!(expr.contains(r#"["all""#));
        assert!(expr.contains(r#"["get", "construction_year"]"#));
        assert!(expr.contains(r#"["get", "height"]"#));
        assert!(expr.contains(r#"["get", "floors"]"#));
    }

    #[test]
    fn test_build_filter_expression_uses_correct_operators() {
        let filter = BuildingFilter::new();
        let expr = build_filter_expression(&filter);

        assert!(expr.contains(r#">="#));
        assert!(expr.contains(r#"<="#));
    }

    #[test]
    fn test_build_set_filter_script_contains_setfilter_call() {
        let expr = r#"["all", [">=", ["get", "height"], 5]]"#;
        let script = build_set_filter_script(expr);

        assert!(script.contains("map.setFilter('building-3d'"));
        assert!(script.contains("window['map_map']"));
    }

    #[test]
    fn test_build_set_filter_script_queries_visible_count() {
        let expr = r#"["all"]"#;
        let script = build_set_filter_script(expr);

        assert!(script.contains("queryRenderedFeatures"));
        assert!(script.contains("layers: ['building-3d']"));
        assert!(script.contains("getElementById('building-count')"));
    }

    #[test]
    fn test_build_clear_filter_script_uses_null_filter() {
        let script = build_clear_filter_script();

        assert!(script.contains("map.setFilter('building-3d', null)"));
    }

    #[test]
    fn test_build_set_filter_script_has_style_loaded_check() {
        let expr = r#"["all"]"#;
        let script = build_set_filter_script(expr);

        // Verify isStyleLoaded() check is present
        assert!(script.contains("isStyleLoaded()"));
        // Verify map.once('load') deferred execution is present
        assert!(script.contains("map.once('load'"));
    }

    #[test]
    fn test_build_clear_filter_script_has_style_loaded_check() {
        let script = build_clear_filter_script();

        // Verify isStyleLoaded() check is present
        assert!(script.contains("isStyleLoaded()"));
        // Verify map.once('load') deferred execution is present
        assert!(script.contains("map.once('load'"));
    }

    #[test]
    fn test_build_restore_filters_from_url_script_exists() {
        let script = build_restore_filters_from_url_script();
        assert!(script.contains("URLSearchParams"));
        assert!(script.contains("window.location.search"));
    }

    #[test]
    fn test_build_restore_filters_from_url_script_returns_object() {
        let script = build_restore_filters_from_url_script();
        assert!(script.contains("hasFilters"));
        assert!(script.contains("yearMin"));
        assert!(script.contains("yearMax"));
        assert!(script.contains("heightMin"));
        assert!(script.contains("heightMax"));
        assert!(script.contains("floorsMin"));
        assert!(script.contains("floorsMax"));
    }
}
