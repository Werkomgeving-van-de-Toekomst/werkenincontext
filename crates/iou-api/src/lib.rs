//! IOU-Modern API Library
//!
//! This library exposes the core types and modules for testing.

pub mod auth;
pub mod db;
pub mod domain_dual_write;
pub mod dual_write;
pub mod etl;
pub mod error;
pub mod middleware;
pub mod migration;
pub mod realtime;
pub mod search_types;
pub mod supabase;
pub mod websockets;

// Re-export commonly used types
pub use db::Database;
pub use dual_write::{DualWrite, DualWriteResult, ReadSource, WriteMode};
pub use etl::{EtlPipeline, EtlConfig, EtlSchedule, EtlMetrics};
pub use middleware::{AuthContext, Role};
pub use migration::{UserMigrator, MigrationReport};
pub use realtime::{RealtimeClient, PresenceTracker};
pub use search_types::{
    AdvancedSearchResult, FacetCount, SearchFacets, SearchParams, SearchMode,
    SortOrder, SuggestionResult, SuggestionType,
};
pub use supabase::SupabasePool;
pub use websockets::types::DocumentStatus;
pub use websockets::limiter::ConnectionLimiter;
