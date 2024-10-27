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
pub struct LoggerMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for LoggerMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
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

pub struct Logger;

impl Logger {
    pub fn new() -> Self {
        Logger
    }
}

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

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LoggerMiddleware {
            service: Rc::new(service),
        }))
    }
}
