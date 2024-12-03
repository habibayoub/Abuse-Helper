//! Data Models and Business Logic Module
//!
//! This module contains all core data structures and their implementations
//! for the application's domain model. It provides a comprehensive set of
//! types and operations for managing application state and business rules.
//!
//! # Module Structure
//!
//! ## Authentication and Authorization
//! * `auth` - JWT tokens, claims, and authentication structures
//!
//! ## Core Business Entities
//! * `customer` - Customer profile and management
//! * `user` - User account management and profiles
//! * `email` - Email processing and storage
//! * `ticket` - Support ticket tracking and management
//!
//! ## Infrastructure
//! * `es` - Elasticsearch integration and search functionality
//! * `nctns` - Notifications system models
//!
//! ## Supporting Structures
//! * `requests` - API request/response structures
//! * `user_log` - User activity logging and audit trails
//!
//! # Features
//! - Type-safe database operations
//! - Serialization support for API responses
//! - Comprehensive error handling
//! - Business logic enforcement
//! - Data validation
//! - Audit logging
//! - Search functionality

/// Authentication and authorization models
pub mod auth;
/// Customer data and operations
pub mod customer;
/// Email processing and management
pub mod email;
/// Elasticsearch integration
pub mod es;
/// Notification system models
pub mod nctns;
/// API request/response structures
pub mod requests;
/// Support ticket management
pub mod ticket;
/// User account management
pub mod user;
/// User activity logging
pub mod user_log;
