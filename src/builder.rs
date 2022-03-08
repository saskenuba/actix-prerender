use crate::middleware::Inner;
use crate::{middleware, PrerenderError, PrerenderMiddleware};
use actix_service::{Service, Transform};
use actix_utils::future;
use actix_utils::future::Ready;
use actix_web::body::{EitherBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
use reqwest::header::HeaderMap;
use reqwest::Client;
use std::rc::Rc;
use url::Url;

/// Builder for Prerender middleware.
///
/// To construct a Prerender middleware, call [`Prerender::build()`] to create a builder.
/// Then you can choose between `self..use_prerender_io`
///
/// # Errors
///
/// TODO
///
/// # Prerender.io example
/// ```
/// use actix_prerender::Prerender;
/// use actix_web::http::header;
///
/// let token = "prerender service token".to_string();
/// let prerender = Prerender::build().use_prerender_io(token);
///
/// // `prerender` can now be used in `App::wrap`.
/// ```
/// # Custom service URL example
/// ```
/// use actix_prerender::Prerender;
/// use actix_web::http::header;
///
/// let token = "prerender service token".to_string();
/// let prerender = Prerender::build().use_custom_prerender_url("https://localhost:5001");
///
/// // `prerender` can now be used in `App::wrap`.
/// ```
#[derive(Clone)]
pub struct Prerender {
    inner: Rc<Inner>,
}

#[derive()]
pub struct PrerenderBuilder {
    pub(crate) forward_headers: bool,
    pub(crate) before_render_fn: Option<fn(&ServiceRequest, &mut HeaderMap)>,
}

fn default_client() -> Client {
    Client::builder()
        .gzip(true)
        .timeout(std::time::Duration::new(25, 0))
        .build()
        .unwrap()
}

impl PrerenderBuilder {
    /// Creates a `Prerender` middleware that delegate requests to the web `prerender.io` service.
    pub fn use_prerender_io(self, token: String) -> Prerender {
        let inner = Inner {
            before_render_fn: self.before_render_fn,
            forward_headers: self.forward_headers,
            inner_client: default_client(),
            prerender_service_url: middleware::prerender_url(),
            prerender_token: Some(token),
        };

        Prerender { inner: Rc::new(inner) }
    }

    /// Creates a `Prerender` middleware that delegates crawler requests to the custom `prerender_service_url`.
    pub fn use_custom_prerender_url(self, prerender_service_url: &str) -> Result<Prerender, PrerenderError> {
        let prerender_service_url = Url::parse(prerender_service_url).map_err(|_| PrerenderError::InvalidUrl)?;

        let inner = Inner {
            before_render_fn: self.before_render_fn,
            forward_headers: self.forward_headers,
            inner_client: default_client(),
            prerender_service_url,
            prerender_token: None,
        };

        Ok(Prerender { inner: Rc::new(inner) })
    }

    /// Allow you to inspect and modify the `HeaderMap` before the request is sent to the `Prerender` service.
    pub fn set_before_render_fn(mut self, prerender_func: fn(req: &ServiceRequest, headers: &mut HeaderMap)) -> Self {
        self.before_render_fn = Some(prerender_func);
        self
    }

    /// Sets the middleware to forward all request headers to the `Prerender` service.
    pub const fn forward_headers(mut self) -> Self {
        self.forward_headers = true;
        self
    }
}

impl Prerender {
    pub fn build() -> PrerenderBuilder {
        PrerenderBuilder {
            forward_headers: false,
            before_render_fn: None,
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for Prerender
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,

    B: MessageBody + 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = PrerenderMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        future::ok(PrerenderMiddleware {
            service,
            inner: Rc::clone(&self.inner),
        })
    }
}
