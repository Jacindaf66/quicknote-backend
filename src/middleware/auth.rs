use actix_web::{
    body::{EitherBody, BoxBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::{LocalBoxFuture, ready, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
// use std::{rc::Rc, task::Context};
use std::rc::Rc;
use crate::middleware::claims::Claims;

// #[derive(Debug, serde::Deserialize)]
// struct Claims {
//     sub: String,
//     exp: usize,
//     user_id: i32,
// }

pub struct AuthMiddleware {
    secret: Rc<String>,
}

impl AuthMiddleware {
    pub fn new(secret: String) -> Self {
        AuthMiddleware {
            secret: Rc::new(secret),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareMiddleware {
            service,
            secret: self.secret.clone(),
        }))
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: S,
    secret: Rc<String>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response=ServiceResponse<B>, Error=Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B, BoxBody>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let secret = self.secret.clone();

        let authorized = if let Some(auth_header) = req.headers().get("Authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if let Some(token) = auth_str.strip_prefix("Bearer ") {
                    let validation = Validation::new(Algorithm::HS256);
                    let decoded = decode::<Claims>(
                        token,
                        &DecodingKey::from_secret(secret.as_bytes()),
                        &validation,
                    );
                    if let Ok(token_data) = decoded {
                        req.extensions_mut().insert(token_data.claims.user_id);
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if !authorized {
            let (req, _pl) = req.into_parts();
            let resp = HttpResponse::Unauthorized()
                .body("Unauthorized: Invalid or missing token")
                .map_into_right_body();
            let response = ServiceResponse::new(req, resp);
            return Box::pin(async { Ok(response) });
        }

        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_left_body())
        })
    }
}
