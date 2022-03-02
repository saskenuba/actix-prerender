use actix_service::{Service, Transform};
use actix_utils::future;
use actix_utils::future::Ready;
use actix_web::body::BoxBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::HeaderMap;
use actix_web::http::uri::PathAndQuery;
use actix_web::http::{header, Method};
use actix_web::{Error, HttpResponse};
use futures_util::future::LocalBoxFuture;
use futures_util::TryFutureExt;
use reqwest::Client;
use std::rc::Rc;
use url::Url;

use crate::errors::PrerenderError;
use crate::{IGNORED_EXTENSIONS, USER_AGENTS};

#[derive(Debug, Clone)]
pub struct Prerender {
    inner: Rc<Inner>,
}

#[derive(Debug, Clone)]
pub struct Inner {
    prerender_service_url: Url,
    inner_client: Client,
    prerender_token: String,
}

impl Prerender {}

#[derive(Debug, Clone)]
pub struct PrerenderBuilder {}

impl PrerenderBuilder {
    pub fn use_prerender_io(self, token: String) -> Prerender {
        let inner = Inner {
            prerender_service_url: prerender_url(),
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
    pub const fn builder() -> PrerenderBuilder {
        PrerenderBuilder {}
    }
}

fn prerender_url() -> Url {
    Url::parse("https://service.prerender.io").unwrap()
}

/// Decides if should prerender the page or not.
///
/// Will NOT prerender on the following cases:
/// * HTTP is not GET or HEAD
/// * User agent is NOT crawler
/// * Is requesting a resource on `IGNORED_EXTENSIONS`
pub(crate) fn should_prerender(req: &ServiceRequest) -> bool {
    let request_headers = req.headers();
    let mut is_crawler = false;

    if ![Method::GET, Method::HEAD].contains(req.method()) {
        return false;
    }

    let req_ua_lowercase = if let Some(user_agent) = request_headers.get(header::USER_AGENT) {
        let user_agent = user_agent.to_str().map(str::to_lowercase);
        if let Ok(ua) = user_agent {
            ua
        } else {
            return false;
        }
    } else {
        return false;
    };

    if USER_AGENTS
        .iter()
        .any(|crawler_ua| req_ua_lowercase.contains(&*crawler_ua.to_lowercase()))
    {
        is_crawler = true;
    }

    // check for ignored extensions
    let is_ignored_extension_url = req.uri().path_and_query().map_or_else(
        || false,
        |path_query| IGNORED_EXTENSIONS.iter().any(|ext| path_query.as_str().contains(ext)),
    );
    if is_ignored_extension_url {
        return false;
    }

    is_crawler
}

#[derive(Debug)]
pub struct PrerenderMiddleware<S> {
    pub(crate) service: S,
    inner: Rc<Inner>,
}

impl<S> PrerenderMiddleware<S> {
    pub fn prepare_build_api_url(service_url: &Url, req: &ServiceRequest) -> String {
        let req_uri = req.uri();
        let req_headers = req.headers();

        let host = req
            .uri()
            .host()
            .or_else(|| req_headers.get("X-Forwarded-Host").and_then(|hdr| hdr.to_str().ok()))
            .or_else(|| req_headers.get(header::HOST).and_then(|hdr| hdr.to_str().ok()))
            .unwrap();

        let scheme = req.uri().scheme_str().unwrap_or("http");
        let url_path_query = req_uri.path_and_query().map(PathAndQuery::as_str).unwrap();

        format!("{}{}://{}{}", service_url, scheme, host, url_path_query)
    }

    pub async fn get_rendered_response(inner: &Inner, req: ServiceRequest) -> Result<ServiceResponse, PrerenderError> {
        let mut prerender_request_headers = HeaderMap::new();
        let forward_headers = true;

        if forward_headers {
            prerender_request_headers = req.headers().clone();
            prerender_request_headers.remove(header::HOST);
        }

        prerender_request_headers.append(header::ACCEPT_ENCODING, "gzip".parse().unwrap());
        prerender_request_headers.append(
            "X-Prerender-Token".parse().unwrap(),
            inner.prerender_token.parse().unwrap(),
        );

        let url_to_request = Self::prepare_build_api_url(&inner.prerender_service_url, &req);
        let prerender_response = inner
            .inner_client
            .get(url_to_request)
            .send()
            .and_then(|resp| resp.bytes())
            .await?;

        let http_response = HttpResponse::Ok().content_type("text/html").body(prerender_response);
        Ok(req.into_response(http_response))
    }
}

impl<S> Service<ServiceRequest> for PrerenderMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<ServiceResponse<BoxBody>, Error>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // life goes on
        if !should_prerender(&req) {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await });
        }

        let inner = Rc::clone(&self.inner);
        Box::pin(async move { Self::get_rendered_response(&inner, req).await.map_err(Into::into) })
    }
}

impl<S> Transform<S, ServiceRequest> for Prerender
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
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

#[cfg(test)]
mod tests {

    use actix_web::http::header;
    use actix_web::middleware::Compat;
    use actix_web::test::TestRequest;
    use actix_web::App;

    use crate::middleware::{prerender_url, should_prerender, Prerender, PrerenderMiddleware};

    fn _init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[actix_web::test]
    async fn compat_compat() {
        App::new().wrap(Compat::new(Prerender::builder().use_prerender_io("".to_string())));
    }

    #[actix_web::test]
    async fn test_human_valid_resource() {
        let req = TestRequest::get()
            .insert_header((
                header::USER_AGENT,
                "Mozilla/5.0 (X11; Linux x86_64; rv:62.0) Gecko/20100101 Firefox/62.0",
            ))
            .uri("http://yourserver.com/clothes/tshirts?query=xl")
            .to_srv_request();

        assert!(!should_prerender(&req));
    }

    #[actix_web::test]
    async fn test_crawler_valid_resource() {
        let req = TestRequest::get()
            .insert_header((
                header::USER_AGENT,
                "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
            ))
            .uri("http://yourserver.com/clothes/tshirts?query=xl")
            .to_srv_request();

        assert!(should_prerender(&req));
    }

    #[actix_web::test]
    async fn test_crawler_ignored_resource() {
        let req = TestRequest::get()
            .insert_header((
                header::USER_AGENT,
                "LinkedInBot/1.0 (compatible; Mozilla/5.0; Jakarta Commons-HttpClient/3.1 +http://www.linkedin.com)",
            ))
            .uri("http://yourserver.com/clothes/tshirts/blue.jpg")
            .to_srv_request();

        let render = should_prerender(&req);
        assert!(!render);
    }

    #[actix_web::test]
    async fn test_crawler_wrong_http_method() {
        let req = TestRequest::post()
            .insert_header((
                header::USER_AGENT,
                "LinkedInBot/1.0 (compatible; Mozilla/5.0; Jakarta Commons-HttpClient/3.1 +http://www.linkedin.com)",
            ))
            .uri("http://yourserver.com/clothes/tshirts/red-dotted")
            .to_srv_request();

        let render = should_prerender(&req);
        assert!(!render);
    }

    fn _create_middleware() -> Prerender {
        Prerender::builder().use_prerender_io("".to_string())
    }

    #[actix_web::test]
    async fn test_redirect_url() {
        let req_url = "http://yourserver.com/clothes/tshirts?query=xl";

        let req = TestRequest::get()
            .insert_header((
                header::USER_AGENT,
                "Mozilla/5.0 (X11; Linux x86_64; rv:62.0) Gecko/20100101 Firefox/62.0",
            ))
            .uri(req_url)
            .to_srv_request();

        // let middleware = create_middleware()
        //     .new_transform(test::ok_service())
        //     .into_inner()
        //     .unwrap();

        assert_eq!(
            PrerenderMiddleware::<()>::prepare_build_api_url(&prerender_url(), &req),
            format!("{}{}", prerender_url(), req_url)
        );
    }
}
