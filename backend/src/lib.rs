//! Backend Service Library
//!
//! Core library providing comprehensive functionality for the backend service:
//!
//! # Module Structure
//!
//! ## Authentication and Authorization
//! - `auth`: Core authentication services
//!   - JWT token management
//!   - User authentication
//!   - Role-based access control
//!
//! ## AI/ML Integration
//! - `llm`: Language Learning Model integration
//!   - AI processing
//!   - Text analysis
//!   - Natural language processing
//!
//! ## Request Processing
//! - `middleware`: Request processing layers
//!   - Authentication middleware
//!   - Logging middleware
//!   - Error handling
//!   - Request validation
//!
//! ## Data Models
//! - `models`: Core data structures
//!   - User models
//!   - Authentication models
//!   - Business logic models
//!   - Database schemas
//!
//! ## Database Integration
//! - `postgres`: Database connectivity
//!   - Connection pooling
//!   - Query execution
//!   - Transaction management
//!
//! ## API Routes
//! - `routes`: HTTP endpoint definitions
//!   - Authentication routes
//!   - Business logic endpoints
//!   - Utility endpoints
//!
//! # Testing
//! - `tests`: Test module (only in test builds)
//!   - Integration tests
//!   - Unit tests
//!   - Mock implementations
//!
//! # Usage
//! The library provides a unified interface through root-level re-exports
//! of commonly used functionality from each module.

// Authentication and Authorization
pub mod auth;
// AI/ML Integration
pub mod llm;
// Request Processing
pub mod middleware;
// Data Models
pub mod models;
// Database Integration
pub mod postgres;
// API Routes
pub mod routes;

// Re-export all the modules at the root level
pub use auth::*;
pub use middleware::*;
pub use postgres::*;
pub use routes::*;

// Testing
#[cfg(test)]
pub mod tests;
