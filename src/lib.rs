//! Prerender for Actix Web

#![forbid(unsafe_code)]
#![deny(nonstandard_style)]
#![allow(clippy::must_use_candidate)]
#![warn(future_incompatible, missing_debug_implementations)]
#![doc(html_logo_url = "https://actix.rs/img/logo.png")]
#![doc(html_favicon_url = "https://actix.rs/favicon.ico")]

use consts::{IGNORED_EXTENSIONS, USER_AGENTS};

mod consts;
pub mod middleware;

// impl<S, B> Service<ServiceRequest> for PrerenderMiddleWare
// where
//     S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
//     S::Future: 'static,
//     B: MessageBody + 'static,
// {
//     type Response = ServiceResponse<EitherBody<B>>;
//     type Error = Error;
//     type Future = LocalBoxFuture<'static, Result<ServiceResponse<EitherBody<B>>, Error>>;
//
//     actix_service::forward_ready!(service);
//
//     fn call(&self, req: ServiceRequest) -> Self::Future {
//         todo!()
//     }
// }
