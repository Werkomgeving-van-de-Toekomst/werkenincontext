//! Real-time Subscription Module
//!
//! Provides integration with Supabase Realtime for WebSocket-based
//! real-time data synchronization.

mod supabase_rt;
mod presence;

pub use supabase_rt::{
    RealtimeClient,
    SubscriptionHandle,
    RealtimeEvent,
    UpdateType,
    RealtimeError,
    RealtimeConfig,
};

pub use presence::{
    PresenceTracker,
    PresenceInfo,
    PresenceStatus,
};
