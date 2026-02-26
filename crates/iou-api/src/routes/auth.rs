//! Authentication endpoints

pub use crate::middleware::auth::{
    login, logout, refresh_token, LoginRequest, LoginResponse, RefreshRequest,
};
