use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use deadpool_postgres::Pool;
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};

use crate::models::auth::Claims;
use crate::models::user_log::UserLog;
use uuid::Uuid;

/// User activity logging middleware service implementation.
///
/// Handles request logging and user activity tracking.
/// Wraps the inner service and performs asynchronous logging after request processing.
///
/// # Usage
/// ```rust
/// app.wrap(Logger::new())
/// ```
pub struct LoggerMiddleware<S> {
    /// Inner service being wrapped - handles the actual request before logging
    service: Rc<S>,
}

/// Service implementation for LoggerMiddleware.
///
/// Implements the core logging logic for incoming requests.
/// Captures request details and user information for audit trails.
///
/// # Type Parameters
/// * `S` - The wrapped service type
/// * `B` - The response body type
impl<S, B> Service<ServiceRequest> for LoggerMiddleware<S>
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

    /// Processes requests and logs user activity asynchronously.
    ///
    /// # Flow
    /// 1. Forwards request to inner service for processing
    /// 2. Extracts user claims from request extensions
    /// 3. Captures request path and response status
    /// 4. Spawns async task for database logging
    ///
    /// # Error Handling
    /// - Logging errors are captured and logged to stderr
    /// - Main request processing continues even if logging fails
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Clone service and extract request data for async block
        let srv = self.service.clone();
        let pool = req
            .app_data::<actix_web::web::Data<Pool>>()
            .unwrap()
            .clone();
        let path = req.path().to_owned();

        Box::pin(async move {
            let res = srv.call(req).await?;

            if let Some(claims) = res.request().extensions().get::<Claims>() {
                let user_uuid: Uuid = claims.sub;
                let action = res.status().as_str().to_owned();

                // Log the action asynchronously
                actix_web::rt::spawn(async move {
                    if let Err(e) = UserLog::create(&pool, user_uuid, &action, &path).await {
                        eprintln!("Failed to log user action: {:?}", e);
                    }
                });
            }

            Ok(res)
        })
    }
}

/// Logging middleware factory.
///
/// Provides a simple interface for adding activity logging middleware
/// to the application pipeline.
///
/// # Example
/// ```rust
/// let logger = Logger::new();
/// app.wrap(logger);
/// ```
pub struct Logger;

impl Logger {
    /// Creates a new Logger middleware instance.
    /// Initializes without any special configuration.
    pub fn new() -> Self {
        Logger
    }
}

/// Transform implementation for Logger middleware.
///
/// Handles the creation and configuration of LoggerMiddleware instances
/// for the Actix service pipeline.
///
/// # Type Parameters
/// * `S` - The service type being transformed
/// * `B` - The response body type
impl<S, B> Transform<S, ServiceRequest> for Logger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = LoggerMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates a new LoggerMiddleware instance wrapping the provided service.
    /// Called by Actix during middleware initialization.
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggerMiddleware {
            service: Rc::new(service),
        }))
    }
}
