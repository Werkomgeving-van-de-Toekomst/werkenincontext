//! Transactional outbox pattern implementation.
//!
//! Ensures reliable data transfer between Supabase and DuckDB by using
//! an outbox table to capture changes atomically with the main transaction.

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Row};
use tracing::{debug, error, info};
use uuid::Uuid;

/// An event in the change outbox
#[derive(Debug, Clone)]
pub struct OutboxEvent {
    pub id: Uuid,
    pub aggregate_type: String,
    pub aggregate_id: Uuid,
    pub event_type: String,
    pub payload: Value,
    pub created_at: DateTime<Utc>,
    pub processed: bool,
    pub processed_at: Option<DateTime<Utc>>,
}

/// Result of processing outbox events
#[derive(Debug, Clone)]
pub struct OutboxProcessResult {
    pub processed_count: usize,
    pub failed_count: usize,
    pub processing_duration_ms: u64,
}

/// Configuration for outbox processor
#[derive(Debug, Clone)]
pub struct OutboxConfig {
    pub batch_size: usize,
    pub max_retries: u32,
    pub retry_delay_ms: u64,
}

impl Default for OutboxConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            max_retries: 3,
            retry_delay_ms: 1000,
        }
    }
}

/// Processes the transactional outbox
pub struct OutboxProcessor {
    pool: PgPool,
    config: OutboxConfig,
}

impl OutboxProcessor {
    /// Create a new outbox processor
    pub fn new(pool: PgPool, batch_size: usize) -> Self {
        Self {
            pool,
            config: OutboxConfig {
                batch_size,
                ..Default::default()
            },
        }
    }

    /// Create with custom config
    pub fn with_config(pool: PgPool, config: OutboxConfig) -> Self {
        Self { pool, config }
    }

    /// Process pending outbox events
    pub async fn process_pending(&self) -> Result<OutboxProcessResult, anyhow::Error> {
        let start = std::time::Instant::now();

        info!("Starting outbox processing batch");

        let rows = sqlx::query(
            r#"
            SELECT id, aggregate_type, aggregate_id, event_type, payload, created_at
            FROM change_outbox
            WHERE processed = false
            ORDER BY created_at ASC
            LIMIT $1
            "#,
        )
        .bind(self.config.batch_size as i64)
        .fetch_all(&self.pool)
        .await?;

        let events: Vec<OutboxEvent> = rows
            .into_iter()
            .map(|row| {
                Ok(OutboxEvent {
                    id: row.try_get("id")?,
                    aggregate_type: row.try_get("aggregate_type")?,
                    aggregate_id: row.try_get("aggregate_id")?,
                    event_type: row.try_get("event_type")?,
                    payload: row.try_get("payload")?,
                    created_at: row.try_get("created_at")?,
                    processed: false,
                    processed_at: None,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;

        if events.is_empty() {
            debug!("No unprocessed events found");
            return Ok(OutboxProcessResult {
                processed_count: 0,
                failed_count: 0,
                processing_duration_ms: start.elapsed().as_millis() as u64,
            });
        }

        info!("Processing {} outbox events", events.len());

        let mut processed_count = 0;
        let mut failed_count = 0;

        for event in events {
            let event_id = event.id;
            debug!("Processing outbox event {}", event_id);

            match self.process_single_event(&event).await {
                Ok(_) => {
                    processed_count += 1;
                    debug!("Successfully processed outbox event {}", event_id);
                }
                Err(e) => {
                    failed_count += 1;
                    error!("Failed to process outbox event {}: {}", event_id, e);

                    // Update retry count
                    if let Err(retry_err) = self.increment_retry_count(event_id).await {
                        error!("Failed to update retry count for event {}: {}", event_id, retry_err);
                    }
                }
            }
        }

        let duration_ms = start.elapsed().as_millis() as u64;
        info!(
            "Outbox processing complete: {} processed, {} failed, {}ms",
            processed_count, failed_count, duration_ms
        );

        Ok(OutboxProcessResult {
            processed_count,
            failed_count,
            processing_duration_ms: duration_ms,
        })
    }

    /// Process a single outbox event
    async fn process_single_event(&self, event: &OutboxEvent) -> Result<(), anyhow::Error> {
        let id = event.id;
        let aggregate_type = event.aggregate_type.as_str();
        let aggregate_id = &event.aggregate_id;
        let event_type = event.event_type.as_str();
        let payload = &event.payload;

        // Step 1: Write to DuckDB (analytics database)
        // This is where the actual data transformation and loading happens
        self.write_to_duckdb(aggregate_type, aggregate_id, event_type, payload)
            .await
            .map_err(|e| {
                error!("DuckDB write failed for event {}: {}", id, e);
                anyhow::anyhow!("DuckDB write failed: {}", e)
            })?;

        sqlx::query(
            r#"
            UPDATE change_outbox
            SET processed = true, processed_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Write event data to DuckDB analytics database
    ///
    /// This method is a placeholder for the actual DuckDB integration.
    /// In production, this would:
    /// 1. Deserialize the payload into the appropriate entity type
    /// 2. Transform the data for analytics schema if needed
    /// 3. Insert/Update the corresponding DuckDB table
    ///
    /// The DuckDB connection would be obtained from the application's
    /// Database pool (typically shared via dependency injection).
    async fn write_to_duckdb(
        &self,
        aggregate_type: &str,
        aggregate_id: &Uuid,
        event_type: &str,
        payload: &Value,
    ) -> Result<(), anyhow::Error> {
        // TODO: Integrate with DuckDB analytics database
        // This requires access to the DuckDB connection pool
        //
        // Example implementation pattern:
        // match aggregate_type {
        //     "information_domain" => {
        //         let domain: InformationDomain = serde_json::from_value(payload.clone())?;
        //         duckdb_conn.execute(
        //             "INSERT OR REPLACE INTO information_domains VALUES (?, ?)",
        //             &[&domain.id, &domain.name]
        //         )?;
        //     }
        //     "document" => { /* similar */ }
        //     _ => warn!("Unknown aggregate type: {}", aggregate_type)
        // }

        debug!(
            "Would write to DuckDB: type={}, id={}, event={}",
            aggregate_type, aggregate_id, event_type
        );

        // For now, this is a no-op that succeeds
        // The DuckDB write will be implemented when the analytics
        // database connection is properly wired through the ETL module
        Ok(())
    }

    /// Increment retry count for a failed event
    async fn increment_retry_count(&self, event_id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE change_outbox
            SET retry_count = retry_count + 1,
                last_error = 'Processing failed'
            WHERE id = $1
            "#,
        )
        .bind(event_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Publish an event to the outbox (called during a transaction)
    pub async fn publish_event(
        &self,
        aggregate_type: &str,
        aggregate_id: Uuid,
        event_type: &str,
        payload: Value,
    ) -> Result<Uuid, sqlx::Error> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO change_outbox (id, aggregate_type, aggregate_id, event_type, payload)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(id)
        .bind(aggregate_type)
        .bind(aggregate_id)
        .bind(event_type)
        .bind(payload)
        .execute(&self.pool)
        .await?;

        debug!(
            "Published outbox event: type={}, id={}, event={}",
            aggregate_type, aggregate_id, event_type
        );

        Ok(id)
    }

    /// Get count of unprocessed events
    pub async fn unprocessed_count(&self) -> Result<i64, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM change_outbox
            WHERE processed = false
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        let count: i64 = row.try_get("count")?;
        Ok(count)
    }

    /// Get outbox statistics
    pub async fn get_stats(&self) -> Result<OutboxStats, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE processed = false) as pending,
                COUNT(*) FILTER (WHERE processed = true) as completed,
                COUNT(*) FILTER (WHERE processed = false AND retry_count > 0) as failed,
                MAX(created_at) FILTER (WHERE processed = false) as oldest_pending
            FROM change_outbox
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(OutboxStats {
            pending: row.try_get::<i64, _>("pending")?,
            completed: row.try_get::<i64, _>("completed")?,
            failed: row.try_get::<i64, _>("failed")?,
            oldest_pending: row.try_get::<Option<DateTime<Utc>>, _>("oldest_pending")?,
        })
    }
}

/// Outbox statistics
#[derive(Debug, Clone)]
pub struct OutboxStats {
    pub pending: i64,
    pub completed: i64,
    pub failed: i64,
    pub oldest_pending: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outbox_config_default() {
        let config = OutboxConfig::default();
        assert_eq!(config.batch_size, 100);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay_ms, 1000);
    }

    #[test]
    fn test_outbox_stats_creation() {
        let stats = OutboxStats {
            pending: 10,
            completed: 100,
            failed: 2,
            oldest_pending: Some(Utc::now()),
        };
        assert_eq!(stats.pending, 10);
        assert_eq!(stats.completed, 100);
    }
}
