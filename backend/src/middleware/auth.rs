use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage,
};
use deadpool_postgres::Pool;
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

use crate::auth::{check_auth, is_authorized};

/// Authentication middleware service implementation.
///
/// Handles request authentication and role-based access control.
/// Wraps the inner service and performs authentication checks before request processing.
///
/// # Usage
/// ```rust
/// app.wrap(Auth::new().role("admin"))
/// ```
pub struct AuthMiddleware<S> {
    /// Inner service being wrapped - handles the actual request after authentication
    service: Rc<S>,
    /// Optional role requirement for authorization - if None, only authentication is performed
    role: Option<String>,
}

/// Service implementation for AuthMiddleware.
///
/// Implements the core authentication and authorization logic for incoming requests.
/// Validates JWT tokens and checks role permissions if specified.
///
/// # Type Parameters
/// * `S` - The wrapped service type
/// * `B` - The response body type
impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    // Delegate readiness check to the inner service
    forward_ready!(service);

    /// Processes incoming requests, performing authentication and authorization checks.
    ///
    /// # Flow
    /// 1. Extracts and validates JWT token from request headers
    /// 2. Checks role permissions if a role requirement is configured
    /// 3. Injects validated claims into request extensions for downstream use
    /// 4. Forwards to inner service if all checks pass
    ///
    /// # Error Handling
    /// - Returns 401 Unauthorized for invalid/missing tokens
    /// - Returns 403 Forbidden for insufficient role permissions
    /// - Propagates database errors during authentication
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Clone service and role for async block ownership
        let srv = self.service.clone();
        let role = self.role.clone();

        Box::pin(async move {
            // Get database pool from application state
            let pool = req.app_data::<web::Data<Pool>>().unwrap().get_ref();

            // Perform authentication check
            match check_auth(&req, pool).await {
                Ok(claims) => {
                    // If role is specified, verify authorization
                    if let Some(required_role) = role {
                        if !is_authorized(&claims, &required_role) {
                            return Err(actix_web::error::ErrorForbidden(
                                "Insufficient permissions",
                            ));
                        }
                    }
                    // Store validated claims for downstream handlers
                    req.extensions_mut().insert(claims);
                    // Forward to inner service
                    let res = srv.call(req).await?;
                    Ok(res)
                }
                Err(e) => Err(e),
            }
        })
    }
}

/// Authentication middleware factory.
///
/// Provides a fluent interface for configuring authentication middleware
/// with optional role-based access control.
///
/// # Example
/// ```rust
/// let auth = Auth::new().role("admin");
/// app.wrap(auth);
/// ```
pub struct Auth {
    /// Required role for accessing protected resources
    /// None means only authentication is required
    role: Option<String>,
}

impl Auth {
    /// Creates a new Auth middleware instance without role requirements.
    /// By default, only validates authentication without role checks.
    pub fn new() -> Self {
        Auth { role: None }
    }

    /// Adds role requirement to the authentication middleware.
    ///
    /// # Arguments
    /// * `role` - Role string that will be required for access
    pub fn role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }
}

/// Transform implementation for Auth middleware.
///
/// Handles the creation and configuration of AuthMiddleware instances
/// for the Actix service pipeline.
///
/// # Type Parameters
/// * `S` - The service type being transformed
/// * `B` - The response body type
impl<S, B> Transform<S, ServiceRequest> for Auth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates a new AuthMiddleware instance wrapping the provided service.
    /// Called by Actix during middleware initialization.
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            role: self.role.clone(),
        }))
    }
}
