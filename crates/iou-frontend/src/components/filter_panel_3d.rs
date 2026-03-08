//! 3D Building Filter Panel Component
//!
//! Provides interactive slider controls for filtering 3D buildings
//! by construction year, height, and floor count.

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
pub fn build_set_filter_script(filter_expr: &str) -> String {
    format!(
        r#"
        (function() {{
            const map = window['map_map'];
            if (!map) {{
                console.error('Map not found for filter update');
                return;
            }}

            try {{
                const filter = {};
                map.setFilter('building-3d', filter);

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

                console.log('Filter updated:', filter, 'Visible buildings:', count);
            }} catch (e) {{
                console.error('Filter update error:', e);
            }}
        }})();
    "#,
        filter_expr
    )
}

/// Builds JavaScript to clear all filters (show all buildings)
pub fn build_clear_filter_script() -> String {
    r#"
        (function() {
            const map = window['map_map'];
            if (!map) {
                console.error('Map not found');
                return;
            }

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
        })();
    "#
    .to_string()
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
                        r#type: "range",
                        min: 1900,
                        max: 2024,
                        value: "{min_year}",
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() {
                                min_year.set(v.min(*max_year.read()));
                            }
                        }
                    }
                    input {
                        class: "filter-slider",
                        r#type: "range",
                        min: 1900,
                        max: 2024,
                        value: "{max_year}",
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() {
                                max_year.set(v.max(*min_year.read()));
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
                        r#type: "range",
                        min: 0,
                        max: 100,
                        step: 1,
                        value: "{min_height}",
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                min_height.set(v.min(*max_height.read()));
                            }
                        }
                    }
                    input {
                        class: "filter-slider",
                        r#type: "range",
                        min: 0,
                        max: 100,
                        step: 1,
                        value: "{max_height}",
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                max_height.set(v.max(*min_height.read()));
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
                        r#type: "range",
                        min: 1,
                        max: 20,
                        step: 1,
                        value: "{min_floors}",
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() {
                                min_floors.set(v.min(*max_floors.read()));
                            }
                        }
                    }
                    input {
                        class: "filter-slider",
                        r#type: "range",
                        min: 1,
                        max: 20,
                        step: 1,
                        value: "{max_floors}",
                        oninput: move |e: Event<FormData>| {
                            if let Ok(v) = e.value().parse::<u32>() {
                                max_floors.set(v.max(*min_floors.read()));
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
}
