//! Alerting configuration and trigger logic.
//!
//! Defines alert thresholds and evaluates metrics against them.

use serde::Serialize;

/// Alert threshold configuration
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub query_p95_ms: f64,
    pub replication_lag_secs: u64,
    pub wal_size_bytes: u64,
    pub disk_free_percent: f64,
    pub auth_failure_rate: f64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            query_p95_ms: 500.0,
            replication_lag_secs: 30,
            wal_size_bytes: 1024 * 1024 * 1024, // 1GB
            disk_free_percent: 20.0,
            auth_failure_rate: 0.01, // 1%
        }
    }
}

/// Represents a triggered alert
#[derive(Debug, Clone, Serialize)]
pub struct Alert {
    pub alert_type: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub current_value: f64,
    pub threshold: f64,
    pub triggered_at: chrono::DateTime<chrono::Utc>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize)]
pub enum AlertSeverity {
    Warning,
    Critical,
}

/// Evaluates metrics against thresholds and triggers alerts
pub struct AlertEngine {
    thresholds: AlertThresholds,
}

impl AlertEngine {
    /// Create a new alert engine with default thresholds
    pub fn new() -> Self {
        Self {
            thresholds: AlertThresholds::default(),
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(thresholds: AlertThresholds) -> Self {
        Self { thresholds }
    }

    /// Evaluate query latency metrics
    pub fn evaluate_query_latency(&self, p95_ms: f64) -> Option<Alert> {
        if p95_ms > self.thresholds.query_p95_ms {
            Some(Alert {
                alert_type: "query_latency".to_string(),
                severity: AlertSeverity::Warning,
                message: format!("Query p95 latency {}ms exceeds threshold {}ms", p95_ms, self.thresholds.query_p95_ms),
                current_value: p95_ms,
                threshold: self.thresholds.query_p95_ms,
                triggered_at: chrono::Utc::now(),
            })
        } else {
            None
        }
    }

    /// Evaluate replication lag
    pub fn evaluate_replication_lag(&self, lag_secs: u64) -> Option<Alert> {
        if lag_secs > self.thresholds.replication_lag_secs {
            Some(Alert {
                alert_type: "replication_lag".to_string(),
                severity: AlertSeverity::Critical,
                message: format!("Replication lag {}s exceeds threshold {}s", lag_secs, self.thresholds.replication_lag_secs),
                current_value: lag_secs as f64,
                threshold: self.thresholds.replication_lag_secs as f64,
                triggered_at: chrono::Utc::now(),
            })
        } else {
            None
        }
    }

    /// Evaluate disk space
    pub fn evaluate_disk_space(&self, free_percent: f64) -> Option<Alert> {
        if free_percent < self.thresholds.disk_free_percent {
            Some(Alert {
                alert_type: "disk_space".to_string(),
                severity: AlertSeverity::Critical,
                message: format!("Disk free {}% is below threshold {}%", free_percent, self.thresholds.disk_free_percent),
                current_value: free_percent,
                threshold: self.thresholds.disk_free_percent,
                triggered_at: chrono::Utc::now(),
            })
        } else {
            None
        }
    }

    /// Evaluate authentication failure rate
    pub fn evaluate_auth_failure_rate(&self, failure_rate: f64) -> Option<Alert> {
        if failure_rate > self.thresholds.auth_failure_rate {
            Some(Alert {
                alert_type: "auth_failure_rate".to_string(),
                severity: AlertSeverity::Critical,
                message: format!("Auth failure rate {:.2}% exceeds threshold {:.2}%", failure_rate * 100.0, self.thresholds.auth_failure_rate * 100.0),
                current_value: failure_rate * 100.0,
                threshold: self.thresholds.auth_failure_rate * 100.0,
                triggered_at: chrono::Utc::now(),
            })
        } else {
            None
        }
    }
}

impl Default for AlertEngine {
    fn default() -> Self {
        Self::new()
    }
}
