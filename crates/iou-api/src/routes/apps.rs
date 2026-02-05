//! App recommendation endpoints

use axum::Json;
use serde::Serialize;
use uuid::Uuid;

use crate::error::ApiError;

/// App info for recommendations
#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub app_type: String,
    pub icon_url: Option<String>,
    pub endpoint_url: String,
    pub relevance_score: f32,
    pub reason: String,
}

/// GET /apps/recommended - Get recommended apps for current context
pub async fn get_recommended_apps() -> Result<Json<Vec<AppInfo>>, ApiError> {
    // TODO: Implement context-aware app recommendations
    // For now, return hardcoded demo apps

    let apps = vec![
        AppInfo {
            id: Uuid::new_v4(),
            name: "Data Verkenner".to_string(),
            description: "Verken en visualiseer provinciale datasets".to_string(),
            app_type: "data_explorer".to_string(),
            icon_url: Some("/icons/data-explorer.svg".to_string()),
            endpoint_url: "/apps/data-verkenner".to_string(),
            relevance_score: 0.95,
            reason: "Populair in vergelijkbare contexten".to_string(),
        },
        AppInfo {
            id: Uuid::new_v4(),
            name: "Document Generator".to_string(),
            description: "Genereer documenten met automatische compliance".to_string(),
            app_type: "document_generator".to_string(),
            icon_url: Some("/icons/document.svg".to_string()),
            endpoint_url: "/apps/document-generator".to_string(),
            relevance_score: 0.90,
            reason: "Vaak gebruikt bij dit type domeinen".to_string(),
        },
        AppInfo {
            id: Uuid::new_v4(),
            name: "Nalevingscontrole".to_string(),
            description: "Monitor Woo, AVG en Archiefwet compliance".to_string(),
            app_type: "compliance_checker".to_string(),
            icon_url: Some("/icons/compliance.svg".to_string()),
            endpoint_url: "/apps/nalevingscontrole".to_string(),
            relevance_score: 0.85,
            reason: "Aanbevolen voor compliance monitoring".to_string(),
        },
        AppInfo {
            id: Uuid::new_v4(),
            name: "Tijdlijn Weergave".to_string(),
            description: "Bekijk de tijdlijn van activiteiten".to_string(),
            app_type: "timeline".to_string(),
            icon_url: Some("/icons/timeline.svg".to_string()),
            endpoint_url: "/apps/tijdlijn-weergave".to_string(),
            relevance_score: 0.80,
            reason: "Handig voor projectoverzicht".to_string(),
        },
        AppInfo {
            id: Uuid::new_v4(),
            name: "Belanghebbenden Kaart".to_string(),
            description: "Visualiseer stakeholders op de kaart".to_string(),
            app_type: "stakeholder_map".to_string(),
            icon_url: Some("/icons/map.svg".to_string()),
            endpoint_url: "/apps/belanghebbenden-kaart".to_string(),
            relevance_score: 0.75,
            reason: "Geschikt voor ruimtelijke projecten".to_string(),
        },
        AppInfo {
            id: Uuid::new_v4(),
            name: "Samenwerkingscentrum".to_string(),
            description: "Werk samen met collega's en externe partijen".to_string(),
            app_type: "collaboration".to_string(),
            icon_url: Some("/icons/collaboration.svg".to_string()),
            endpoint_url: "/apps/samenwerkingscentrum".to_string(),
            relevance_score: 0.70,
            reason: "Voor projecten met meerdere betrokkenen".to_string(),
        },
        AppInfo {
            id: Uuid::new_v4(),
            name: "GraphRAG Explorer".to_string(),
            description: "Ontdek relaties via de kennisgraaf".to_string(),
            app_type: "graph_explorer".to_string(),
            icon_url: Some("/icons/graph.svg".to_string()),
            endpoint_url: "/apps/graphrag-explorer".to_string(),
            relevance_score: 0.65,
            reason: "Ontdek verborgen verbanden".to_string(),
        },
    ];

    Ok(Json(apps))
}
