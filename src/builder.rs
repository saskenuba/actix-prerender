use crate::middleware::Inner;
use crate::{middleware, PrerenderError, PrerenderMiddleware};
use actix_service::{Service, Transform};
use actix_utils::future;
use actix_utils::future::Ready;
use actix_web::body::{EitherBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::Error;
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
/// let prerender = Prerender::build().use_custom_prerender_url("https://localhost:5001", token);
///
/// // `prerender` can now be used in `App::wrap`.
/// ```
#[derive(Debug, Clone)]
pub struct Prerender {
    inner: Rc<Inner>,
}

#[derive(Debug, Clone)]
pub struct PrerenderBuilder {}

impl PrerenderBuilder {
    pub fn use_prerender_io(self, token: String) -> Prerender {
        let inner = Inner {
            prerender_service_url: middleware::prerender_url(),
            inner_client: Client::default(),
            prerender_token: token,
        };

        Prerender { inner: Rc::new(inner) }
    }

    pub fn use_custom_prerender_url(
        self,
        prerender_service_url: &str,
        token: String,
    ) -> Result<Prerender, PrerenderError> {
        let prerender_service_url = Url::parse(prerender_service_url).map_err(|_| PrerenderError::InvalidUrl)?;

        let inner = Inner {
            prerender_service_url,
            inner_client: Client::default(),
            prerender_token: token,
        };

        Ok(Prerender { inner: Rc::new(inner) })
    }
}

impl Prerender {
    pub const fn build() -> PrerenderBuilder {
        PrerenderBuilder {}
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
