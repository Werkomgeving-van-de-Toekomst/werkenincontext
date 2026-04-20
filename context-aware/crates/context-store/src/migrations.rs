// =============================================================================
// Database Migrations for Context-Aware Data
// =============================================================================

use arangors::Client;
use anyhow::Result;

use crate::StoreConfig;

/// Run all pending migrations
pub async fn migrate(config: &StoreConfig) -> Result<()> {
    let client = Client::new_with_auth(&config.url, &config.username, &config.password)?;
    let db = client.db(&config.database).await?;

    // Create collections if they don't exist
    create_collections(&db).await?;

    // Create indexes
    create_indexes(&db).await?;

    // Create edge collections for relationships
    create_edge_collections(&db).await?;

    tracing::info!("Context-Aware database migrations completed");
    Ok(())
}

async fn create_collections(db: &arangors::Database<arangors::client::reqwest::ReqwestClient>) -> Result<()> {
    let collections = vec![
        ("context", false),
        ("context_record", false),
        ("context_type", true),
        ("context_inference", false),
        ("context_quality", false),
    ];

    for (name, is_system) in collections {
        if !db.collection_exists(name).await? {
            db.create_collection(name)
                .is_system(is_system)
                .await?;
            tracing::info!("Created collection: {}", name);
        }
    }

    Ok(())
}

async fn create_indexes(db: &arangors::Database<arangors::client::reqwest::ReqwestClient>) -> Result<()> {
    // Context collection indexes
    let context = db.collection("context").await?;

    // Primary index on informatieobject_id
    context.create_hash_index(vec!["informatieobject_id"])
        .unique(false)
        .sparse(false)
        .await?;

    // Index on organisatie_id
    context.create_hash_index(vec!["organisatie_id"])
        .unique(false)
        .sparse(false)
        .await?;

    // Index on domain
    context.create_hash_index(vec!["domain.primair_domein"])
        .unique(false)
        .sparse(true)
        .await?;

    // Index on validity period
    context.create_skiplist_index(vec!["geldigheid.begindatum", "geldigheid.einddatum"])
        .unique(false)
        .sparse(true)
        .await?;

    // Full-text index on semantic keywords
    context.create_fulltext_index("semantic.trefwoorden", 3)
        .await?;

    // Context record indexes
    let record = db.collection("context_record").await?;

    record.create_hash_index(vec!["context_id"])
        .unique(false)
        .sparse(false)
        .await?;

    record.create_hash_index(vec!["context_id", "versie"])
        .unique(true)
        .sparse(false)
        .await?;

    tracing::info!("Created indexes");
    Ok(())
}

async fn create_edge_collections(db: &arangors::Database<arangors::client::reqwest::ReqwestClient>) -> Result<()> {
    let edge_collections = vec![
        ("has_context", "context", "informatieobject"),
        ("has_type", "context", "context_type"),
        ("inferred_by", "context", "context_inference"),
        ("has_quality", "context", "context_quality"),
    ];

    for (name, _from, _to) in edge_collections {
        if !db.collection_exists(name).await? {
            db.create_edge_collection(name).await?;
            tracing::info!("Created edge collection: {}", name);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_migrations() {
        let config = StoreConfig::default();
        migrate(&config).await.unwrap();
    }
}
