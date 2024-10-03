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

pub struct AuthMiddleware<S> {
    service: Rc<S>,
    role: Option<String>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddleware<S>
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
        let role = self.role.clone();

        Box::pin(async move {
            let pool = req.app_data::<web::Data<Pool>>().unwrap().get_ref();
            match check_auth(&req, pool).await {
                Ok(claims) => {
                    if let Some(required_role) = role {
                        if !is_authorized(&claims, &required_role) {
                            return Err(actix_web::error::ErrorForbidden(
                                "Insufficient permissions",
                            ));
                        }
                    }
                    req.extensions_mut().insert(claims);
                    let res = srv.call(req).await?;
                    Ok(res)
                }
                Err(e) => Err(e),
            }
        })
    }
}

pub struct Auth {
    role: Option<String>,
}

impl Auth {
    pub fn new() -> Self {
        Auth { role: None }
    }

    pub fn role(mut self, role: &str) -> Self {
        self.role = Some(role.to_string());
        self
    }
}

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

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            role: self.role.clone(),
        }))
    }
}
