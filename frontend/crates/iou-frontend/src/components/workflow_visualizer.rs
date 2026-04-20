//! Interactive workflow visualizer component
//!
//! Displays workflows as an SVG-based directed graph with:
//! - Stage nodes with status-based coloring
//! - Pan/zoom for large workflows
//! - Click handlers for stage details
//! - Real-time updates via WebSocket

use dioxus::prelude::*;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use crate::components::workflow_stage_tracker::{StageView, StageStatus};

/// Workflow visualizer component
#[component]
pub fn WorkflowVisualizer(
    document_id: Uuid,
    #[props(default)] stages: Vec<StageView>,
    #[props(default)] on_stage_click: Option<EventHandler<String>>,
) -> Element {
    let zoom = use_signal(|| 1.0);
    let pan_x = use_signal(|| 0.0);
    let pan_y = use_signal(|| 0.0);
    let is_dragging = use_signal(|| false);
    let drag_start = use_signal(|| (0.0, 0.0));
    let selected_stage = use_signal(|| None::<String>);

    // Calculate layout positions for stages
    let layout = use_signal(|| calculate_layout(&stages));

    // Re-calculate layout when stages change
    use_effect(move || {
        layout.set(calculate_layout(&stages));
    });

    let zoom_in = {
        let zoom = zoom.clone();
        move |_| {
            zoom.modify(|z| (z * 1.2).min(3.0));
        }
    };

    let zoom_out = {
        let zoom = zoom.clone();
        move |_| {
            zoom.modify(|z| (z / 1.2).max(0.3));
        }
    };

    let reset_view = {
        let zoom = zoom.clone();
        let pan_x = pan_x.clone();
        let pan_y = pan_y.clone();
        move |_| {
            zoom.set(1.0);
            pan_x.set(0.0);
            pan_y.set(0.0);
        }
    };

    let handle_stage_click = {
        let on_stage_click = on_stage_click.clone();
        let selected_stage = selected_stage.clone();
        move |stage_id: String| {
            selected_stage.set(Some(stage_id.clone()));
            if let Some(ref handler) = on_stage_click {
                handler.call(stage_id);
            }
        }
    };

    rsx! {
        div { class: "workflow-visualizer-container",
            // Toolbar
            div { class: "visualizer-toolbar",
                button {
                    class: "toolbar-btn",
                    onclick: zoom_in,
                    svg {
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        "stroke-width": "2",
                        path { d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM10 7v3m0 0v3m0-3h3m-3 0H7" }
                    }
                }
                button {
                    class: "toolbar-btn",
                    onclick: zoom_out,
                    svg {
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        "stroke-width": "2",
                        path { d: "M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM13 10H7" }
                    }
                }
                button {
                    class: "toolbar-btn",
                    onclick: reset_view,
                    svg {
                        view_box: "0 0 24 24",
                        fill: "none",
                        stroke: "currentColor",
                        "stroke-width": "2",
                        path { d: "M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" }
                    }
                }
                div { class: "zoom-level", "Zoom: {(*zoom * 100.0) as i32}%" }
            }

            // SVG visualization
            svg {
                class: "workflow-graph",
                preserve_aspect_ratio: "xMidYMid meet",
                viewBox: "0 0 1200 400",
                onmousedown: move |evt| {
                    is_dragging.set(true);
                    drag_start.set((evt.client_x(), evt.client_y()));
                },
                onmouseup: move |_| { is_dragging.set(false); },
                onmouseleave: move |_| { is_dragging.set(false); },
                onmousemove: move |evt| {
                    if *is_dragging.current() {
                        let (start_x, start_y) = *drag_start.current();
                        let dx = evt.client_x() - start_x;
                        let dy = evt.client_y() - start_y;
                        pan_x.with_mut(|x| *x += dx / *zoom.current());
                        pan_y.with_mut(|y| *y += dy / *zoom.current());
                        drag_start.set((evt.client_x(), evt.client_y()));
                    }
                },

                // Definitions for markers and filters
                defs {
                    marker {
                        id: "arrowhead",
                        "marker-width": "10",
                        "marker-height": "7",
                        "refX": "9",
                        "refY": "3.5",
                        "orient": "auto",
                        polygon {
                            points: "0 0, 10 3.5, 0 7",
                            fill: "#94a3b8",
                        }
                    }
                    filter {
                        id: "shadow",
                        feDropShadow {
                            dx: "2",
                            dy: "2",
                            std_deviation: "3",
                            "flood-opacity": "0.2",
                        }
                    }
                }

                // Group with pan/zoom transform
                g {
                    transform: "translate({pan_x} {pan_y}) scale({zoom})",

                    // Connection arrows
                    {layout.connections.iter().map(|conn| rsx! {
                        path {
                            class: "workflow-connection",
                            d: "{conn.path}",
                            "marker-end": "url(#arrowhead)",
                            stroke: if conn.has_active { "#0066CC" } else { "#cbd5e1" },
                            "stroke-width": if conn.has_active { "3" } else { "2" },
                            "stroke-dasharray": if conn.has_active { "none" } else { "5,5" },
                        }
                    })}

                    // Stage nodes
                    {layout.stages.iter().map(|stage_layout| {
                        let stage = stages.iter().find(|s| s.stage_id == stage_layout.id);
                        let status = stage.map(|s| s.status).unwrap_or(StageStatus::Pending);
                        let stage_name = stage.map(|s| s.stage_name.clone()).unwrap_or_default();
                        let is_selected = selected_stage.current().as_ref() == Some(&stage_layout.id);

                        rsx!(
                            g {
                                class: if is_selected { "stage-node selected" } else { "stage-node" },
                                transform: "translate({stage_layout.x} {stage_layout.y})",
                                onclick: move |_| handle_stage_click(stage_layout.id.clone()),
                                cursor: "pointer",

                                // Node shadow
                                rect {
                                    x: "-75",
                                    y: "-25",
                                    width: "150",
                                    height: "50",
                                    rx: "8",
                                    fill: get_stage_color(status),
                                    filter: "url(#shadow)",
                                }

                                // Status indicator
                                circle {
                                    cx: "-60",
                                    cy: "0",
                                    r: "6",
                                    fill: get_status_indicator_color(status),
                                }

                                // Stage name
                                text {
                                    x: "0",
                                    y: "5",
                                    "text-anchor": "middle",
                                    fill: "white",
                                    "font-size": "14",
                                    "font-weight": "600",
                                    "{stage_name}"
                                }

                                // Deadline badge if applicable
                                {stage.and_then(|s| s.deadline).map(|deadline| {
                                    let hours_remaining = (deadline.signed_duration_since(Utc::now()).num_minutes() / 60) as f64;
                                    let urgency = if hours_remaining < 0.0 { "overdue" }
                                        else if hours_remaining < 12.0 { "critical" }
                                        else if hours_remaining < 24.0 { "warning" }
                                        else { "healthy" };

                                    rsx!(
                                        g {
                                            transform: "translate(60 -20)",
                                            rect {
                                                x: "-25",
                                                y: "-35",
                                                width: "50",
                                                height: "20",
                                                rx: "4",
                                                fill: get_urgency_color(urgency),
                                            }
                                            text {
                                                x: "0",
                                                y: "-21",
                                                "text-anchor": "middle",
                                                fill: "white",
                                                "font-size": "10",
                                                "{format_duration(hours_remaining)}"
                                            }
                                        }
                                    )
                                })}
                            }
                        )
                    })}
                }
            }

            // Legend
            div { class: "visualizer-legend",
                h4 { "Status Legend" }
                div { class: "legend-items",
                    div { class: "legend-item",
                        div { class: "legend-color", style: "background: #94a3b8;" }
                        span { "Pending" }
                    }
                    div { class: "legend-item",
                        div { class: "legend-color", style: "background: #0066CC;" }
                        span { "In Progress" }
                    }
                    div { class: "legend-item",
                        div { class: "legend-color", style: "background: #22c55e;" }
                        span { "Completed" }
                    }
                    div { class: "legend-item",
                        div { class: "legend-color", style: "background: #ef4444;" }
                        span { "Rejected/Expired" }
                    }
                    div { class: "legend-item",
                        div { class: "legend-color", style: "background: #f97316;" }
                        span { "Deadline Warning" }
                    }
                }
            }
        }
    }
}

/// Calculate layout positions for stages
fn calculate_layout(stages: &[StageView]) -> GraphLayout {
    if stages.is_empty() {
        return GraphLayout { stages: vec![], connections: vec![] };
    }

    let stage_count = stages.len();
    let width = 1200.0;
    let height = 400.0;
    let node_spacing = width / (stage_count + 1) as f64;

    let stage_nodes: Vec<StageNodeLayout> = stages.iter().enumerate().map(|(i, stage)| {
        StageNodeLayout {
            id: stage.stage_id.clone(),
            x: node_spacing * (i + 1) as f64,
            y: height / 2.0,
        }
    }).collect();

    let connections: Vec<ConnectionLayout> = stage_nodes.windows(2).enumerate().map(|(i, nodes)| {
        ConnectionLayout {
            path: format!("M {} {} L {} {}", nodes[0].x, nodes[0].y, nodes[1].x, nodes[1].y),
            has_active: stages.get(i).map(|s| matches!(s.status, StageStatus::InProgress)).unwrap_or(false),
        }
    }).collect();

    GraphLayout { stages: stage_nodes, connections }
}

/// Get stage background color based on status
fn get_stage_color(status: StageStatus) -> &'static str {
    match status {
        StageStatus::Pending => "#94a3b8",
        StageStatus::InProgress => "#0066CC",
        StageStatus::Completed => "#22c55e",
        StageStatus::Skipped => "#e2e8f0",
        StageStatus::Expired => "#ef4444",
    }
}

/// Get status indicator color
fn get_status_indicator_color(status: StageStatus) -> &'static str {
    match status {
        StageStatus::Pending => "#cbd5e1",
        StageStatus::InProgress => "#60a5fa",
        StageStatus::Completed => "#86efac",
        StageStatus::Skipped => "#f1f5f9",
        StageStatus::Expired => "#fca5a5",
    }
}

/// Get urgency color for deadline badges
fn get_urgency_color(urgency: &str) -> &'static str {
    match urgency {
        "overdue" => "#dc2626",
        "critical" => "#ea580c",
        "warning" => "#f59e0b",
        _ => "#22c55e",
    }
}

/// Format duration for display
fn format_duration(hours: f64) -> String {
    if hours < 0.0 {
        format!("{}d overdue", (-hours / 24.0).ceil() as i32)
    } else if hours < 1.0 {
        format!("{}m", (hours * 60.0).ceil() as i32)
    } else if hours < 24.0 {
        format!("{}h", hours.ceil() as i32)
    } else {
        format!("{}d", (hours / 24.0).ceil() as i32)
    }
}

// ============================================================================
// Types
// ============================================================================

struct GraphLayout {
    stages: Vec<StageNodeLayout>,
    connections: Vec<ConnectionLayout>,
}

struct StageNodeLayout {
    id: String,
    x: f64,
    y: f64,
}

struct ConnectionLayout {
    path: String,
    has_active: bool,
}
