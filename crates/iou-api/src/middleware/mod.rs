//! Middleware for the IOU-Modern API

pub mod auth;

pub use auth::{
    auth_middleware, optional_auth_middleware, AuthContext, require_permission,
};
