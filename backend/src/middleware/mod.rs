//! Application Middleware Module
//!
//! Provides essential middleware components for request processing, security,
//! and monitoring. This module centralizes all middleware implementations
//! used throughout the application.
//!
//! # Architecture
//! The middleware stack consists of:
//! - Authentication & Authorization (auth)
//! - Activity Logging (logger)
//!
//! # Features
//! - JWT-based authentication
//! - Role-based access control
//! - Async request logging
//! - User activity tracking
//!
//! # Usage
//! ```rust
//! use actix_web::App;
//! use crate::middleware::{Auth, Logger};
//!
//! fn configure_app(app: App) -> App {
//!     app.wrap(Auth::new().role("admin"))
//!        .wrap(Logger::new())
//! }
//! ```
//!
//! # Order of Execution
//! 1. Authentication middleware validates requests
//! 2. Logger middleware tracks successful requests
//!
//! # Module Structure

/// Authentication and authorization middleware implementation
mod auth;
/// Request logging and activity tracking middleware
mod logger;

// Public exports for application use
pub use auth::Auth;
pub use logger::Logger;
