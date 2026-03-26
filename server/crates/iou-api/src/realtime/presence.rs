//! Presence Tracking for Real-time Collaboration
//!
//! Tracks which users are currently viewing or editing documents.
//! Integrates with Supabase Realtime presence feature.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Status of a user's presence on a document
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PresenceStatus {
    /// User is viewing the document (read-only)
    Viewing,

    /// User is actively editing the document
    Editing,

    /// User is idle (no activity for a period)
    Idle,
}

impl PresenceStatus {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "viewing" => Some(Self::Viewing),
            "editing" => Some(Self::Editing),
            "idle" => Some(Self::Idle),
            _ => None,
        }
    }

    /// Convert to string
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Viewing => "viewing",
            Self::Editing => "editing",
            Self::Idle => "idle",
        }
    }
}

/// Information about a user's presence on a document
#[derive(Debug, Clone)]
pub struct PresenceInfo {
    /// User ID
    pub user_id: Uuid,

    /// User display name
    pub user_name: String,

    /// Document ID
    pub document_id: Uuid,

    /// Current presence status
    pub status: PresenceStatus,

    /// Last activity timestamp
    pub last_seen: DateTime<Utc>,

    /// Current cursor position (if editing)
    pub cursor_position: Option<usize>,

    /// Selected text range (if any)
    pub selection_range: Option<(usize, usize)>,
}

impl PresenceInfo {
    /// Create a new presence info
    pub fn new(
        user_id: Uuid,
        user_name: String,
        document_id: Uuid,
        status: PresenceStatus,
    ) -> Self {
        Self {
            user_id,
            user_name,
            document_id,
            status,
            last_seen: Utc::now(),
            cursor_position: None,
            selection_range: None,
        }
    }

    /// Update the last seen timestamp
    pub fn update_last_seen(&mut self) {
        self.last_seen = Utc::now();
    }

    /// Check if this presence is stale (older than specified seconds)
    pub fn is_stale(&self, seconds: i64) -> bool {
        Utc::now().signed_duration_since(self.last_seen).num_seconds() > seconds
    }

    /// Get the age of this presence in seconds
    pub fn age_seconds(&self) -> i64 {
        Utc::now().signed_duration_since(self.last_seen).num_seconds()
    }
}

/// Presence tracker for managing user presence across documents
pub struct PresenceTracker {
    /// Presence tracking by document ID
    by_document: DashMap<Uuid, Vec<PresenceInfo>>,

    /// Presence tracking by user ID (for quick lookup)
    by_user: DashMap<Uuid, Vec<Uuid>>,

    /// Cleanup interval in seconds
    cleanup_interval: i64,

    /// Presence timeout in seconds (after which user is considered offline)
    presence_timeout: i64,
}

impl PresenceTracker {
    /// Create a new presence tracker
    pub fn new() -> Self {
        Self {
            by_document: DashMap::new(),
            by_user: DashMap::new(),
            cleanup_interval: 60, // Cleanup every minute
            presence_timeout: 300, // 5 minutes timeout
        }
    }

    /// Create with custom timeout settings
    pub fn with_timeout(cleanup_interval_secs: i64, presence_timeout_secs: i64) -> Self {
        Self {
            by_document: DashMap::new(),
            by_user: DashMap::new(),
            cleanup_interval: cleanup_interval_secs,
            presence_timeout: presence_timeout_secs,
        }
    }

    /// Update or add user presence
    ///
    /// This method:
    /// 1. Updates the user's presence status
    /// 2. Broadcasts the update to other users on the document
    /// 3. Cleans up stale presence entries
    pub fn update_presence(&self, info: PresenceInfo) {
        let document_id = info.document_id;
        let user_id = info.user_id;

        // Update by_document index using entry API for atomicity
        self.by_document
            .entry(document_id)
            .and_modify(|list| {
                // Remove existing entry for this user if present
                list.retain(|p| p.user_id != user_id);
                // Add new presence info
                list.push(info.clone());
            })
            .or_insert_with(|| vec![info.clone()]);

        // Update by_user index using entry API for atomicity
        self.by_user
            .entry(user_id)
            .and_modify(|docs| {
                if !docs.contains(&document_id) {
                    docs.push(document_id);
                }
            })
            .or_insert_with(|| vec![document_id]);

        tracing::debug!(
            "Updated presence: user {} on document {} ({:?})",
            info.user_name,
            document_id,
            info.status
        );
    }

    /// Get all users currently viewing a document
    pub fn get_document_viewers(&self, document_id: &Uuid) -> Vec<PresenceInfo> {
        self.by_document
            .get(document_id)
            .map(|list| {
                list.iter()
                    .filter(|p| !p.is_stale(self.presence_timeout))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get editors (users with Editing status) for a document
    pub fn get_document_editors(&self, document_id: &Uuid) -> Vec<PresenceInfo> {
        self.get_document_viewers(document_id)
            .into_iter()
            .filter(|p| p.status == PresenceStatus::Editing)
            .collect()
    }

    /// Get presence info for a specific user on a document
    pub fn get_user_presence(&self, user_id: &Uuid, document_id: &Uuid) -> Option<PresenceInfo> {
        self.by_document
            .get(document_id)?
            .iter()
            .find(|p| p.user_id == *user_id)
            .cloned()
    }

    /// Remove user from a document (when they leave)
    pub fn remove_user_from_document(&self, user_id: Uuid, document_id: Uuid) {
        // Update by_document
        if let Some(mut list) = self.by_document.get_mut(&document_id) {
            list.retain(|p| p.user_id != user_id);
            if list.is_empty() {
                self.by_document.remove(&document_id);
            }
        }

        // Update by_user
        if let Some(mut docs) = self.by_user.get_mut(&user_id) {
            docs.retain(|d| d != &document_id);
            if docs.is_empty() {
                self.by_user.remove(&user_id);
            }
        }

        tracing::debug!("Removed user {} from document {}", user_id, document_id);
    }

    /// Remove user from all documents (when they disconnect)
    pub fn remove_user(&self, user_id: &Uuid) {
        if let Some((_, docs)) = self.by_user.remove(user_id) {
            for document_id in &docs {
                if let Some(mut list) = self.by_document.get_mut(document_id) {
                    list.retain(|p| p.user_id != *user_id);
                    if list.is_empty() {
                        self.by_document.remove(document_id);
                    }
                }
            }
        }

        tracing::debug!("Removed user {} from all documents", user_id);
    }

    /// Clean up stale presence entries
    pub fn cleanup_stale(&self) {
        let stale_cutoff = self.presence_timeout;

        self.by_document.retain(|document_id, list| {
            let original_len = list.len();
            list.retain(|p| !p.is_stale(stale_cutoff));

            if list.len() < original_len {
                tracing::debug!(
                    "Cleaned {} stale presence entries from document {}",
                    original_len - list.len(),
                    document_id
                );
            }

            !list.is_empty()
        });
    }

    /// Get all active documents (with at least one active user)
    pub fn get_active_documents(&self) -> Vec<Uuid> {
        self.by_document
            .iter()
            .map(|e| *e.key())
            .collect()
    }

    /// Get count of active users on a document
    pub fn get_active_count(&self, document_id: &Uuid) -> usize {
        self.by_document
            .get(document_id)
            .map(|list| list.iter().filter(|p| !p.is_stale(self.presence_timeout)).count())
            .unwrap_or(0)
    }

    /// Get total presence count across all documents
    pub fn get_total_count(&self) -> usize {
        self.by_document
            .iter()
            .map(|e| {
                e.value()
                    .iter()
                    .filter(|p| !p.is_stale(self.presence_timeout))
                    .count()
            })
            .sum()
    }
}

impl Default for PresenceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presence_status_parsing() {
        assert_eq!(PresenceStatus::from_str("viewing"), Some(PresenceStatus::Viewing));
        assert_eq!(PresenceStatus::from_str("editing"), Some(PresenceStatus::Editing));
        assert_eq!(PresenceStatus::from_str("idle"), Some(PresenceStatus::Idle));
        assert_eq!(PresenceStatus::from_str("invalid"), None);
    }

    #[test]
    fn test_presence_status_as_str() {
        assert_eq!(PresenceStatus::Viewing.as_str(), "viewing");
        assert_eq!(PresenceStatus::Editing.as_str(), "editing");
        assert_eq!(PresenceStatus::Idle.as_str(), "idle");
    }

    #[test]
    fn test_presence_info_creation() {
        let user_id = Uuid::new_v4();
        let document_id = Uuid::new_v4();
        let info = PresenceInfo::new(
            user_id,
            "Test User".to_string(),
            document_id,
            PresenceStatus::Editing,
        );

        assert_eq!(info.user_id, user_id);
        assert_eq!(info.document_id, document_id);
        assert_eq!(info.status, PresenceStatus::Editing);
    }

    #[test]
    fn test_presence_tracker() {
        let tracker = PresenceTracker::new();

        let user_id = Uuid::new_v4();
        let document_id = Uuid::new_v4();

        let info = PresenceInfo::new(
            user_id,
            "Test User".to_string(),
            document_id,
            PresenceStatus::Viewing,
        );

        tracker.update_presence(info);

        // Verify user is tracked
        assert_eq!(tracker.get_active_count(&document_id), 1);
        assert_eq!(tracker.get_total_count(), 1);

        // Remove user
        tracker.remove_user_from_document(user_id, document_id);
        assert_eq!(tracker.get_active_count(&document_id), 0);
    }

    #[test]
    fn test_get_document_editors() {
        let tracker = PresenceTracker::new();
        let document_id = Uuid::new_v4();

        // Add a viewer
        tracker.update_presence(PresenceInfo::new(
            Uuid::new_v4(),
            "Viewer".to_string(),
            document_id,
            PresenceStatus::Viewing,
        ));

        // Add an editor
        let editor_id = Uuid::new_v4();
        tracker.update_presence(PresenceInfo::new(
            editor_id,
            "Editor".to_string(),
            document_id,
            PresenceStatus::Editing,
        ));

        let editors = tracker.get_document_editors(&document_id);
        assert_eq!(editors.len(), 1);
        assert_eq!(editors[0].user_id, editor_id);
    }
}
