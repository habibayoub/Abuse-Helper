//! Route Module Collection
//!
//! Centralizes all application routing modules and their functionalities:
//!
//! # Modules
//! - `auth`: Authentication and authorization routes
//!   - Login/logout
//!   - Token management
//!   - User creation
//!   - Permission handling
//!
//! - `config`: Route configuration and middleware setup
//!   - Route registration
//!   - Middleware chains
//!   - Security configurations
//!   - Health checks
//!
//! - `customer`: Customer management endpoints
//!   - Customer listing
//!   - Customer lookup
//!   - Profile management
//!   - Data access
//!
//! - `email`: Email processing and management
//!   - Email sending
//!   - Processing queues
//!   - Ticket associations
//!   - Search functionality
//!
//! - `nctns`: Network and Cyber Threat Notification System
//!   - Threat notifications
//!   - Security alerts
//!   - Incident tracking
//!   - Threat analysis
//!
//! - `ticket`: Ticket management system
//!   - Ticket creation
//!   - Status updates
//!   - Email linking
//!   - Search operations
//!
//! - `util`: Utility endpoints
//!   - WHOIS lookups
//!   - System diagnostics
//!   - Helper functions
//!   - Support tools
//!
//! # Security Considerations
//! - All routes implement appropriate authentication
//! - Role-based access control
//! - Input validation
//! - Rate limiting
//!
//! # Usage
//! Each module can be imported separately or accessed through this parent module.
//! Routes are configured in the `config` module and registered in the application
//! startup.

pub mod auth;
pub mod config;
pub mod customer;
pub mod email;
pub mod nctns;
pub mod ticket;
pub mod util;
