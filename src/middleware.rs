use std::rc::Rc;

use actix_service::Service;
use actix_web::body::{EitherBody, MessageBody};
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::http::header::ContentEncoding;
use actix_web::http::uri::PathAndQuery;
use actix_web::http::{header, Method};
use actix_web::{Error, HttpResponse};
use futures_util::future::LocalBoxFuture;
use futures_util::TryFutureExt;
use log::trace;
use reqwest::header::HeaderMap;
use reqwest::Client;
use url::Url;

use crate::error::PrerenderError;
use crate::{IGNORED_EXTENSIONS, USER_AGENTS};

pub struct Inner {
    pub(crate) inner_client: Client,

    pub(crate) forward_headers: bool,
    pub(crate) before_render_fn: Option<fn(&ServiceRequest, &mut HeaderMap)>,

    pub(crate) prerender_service_url: Url,
    pub(crate) prerender_token: Option<String>,
}

pub(crate) fn prerender_url() -> Url {
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

#[derive()]
pub struct PrerenderMiddleware<S> {
    pub(crate) service: S,
    pub(crate) inner: Rc<Inner>,
}

impl<S> PrerenderMiddleware<S> {
    pub fn prepare_build_api_url(service_url: &Url, req: &ServiceRequest) -> String {
        let req_uri = req.uri();
        let req_headers = req.headers();

        let mut scheme = req.uri().scheme_str().unwrap_or("http");

        // handle visitors using Cloudflare Flexible SSL
        if let Some(Ok(hdr_value)) = req_headers.get("cf-visitor").map(|val| val.to_str()) {
            let index = hdr_value.rmatch_indices("http").collect::<Vec<_>>().remove(0).0;
            scheme = &hdr_value[index..hdr_value.len() - 1];
        }

        if let Some(Ok(hdr_value)) = req_headers.get("X-Forwarded-Proto").map(|val| val.to_str()) {
            scheme = hdr_value.split(',').collect::<Vec<_>>().remove(0);
        }

        let host = req
            .uri()
            .host()
            .or_else(|| req_headers.get("X-Forwarded-Host").and_then(|hdr| hdr.to_str().ok()))
            .or_else(|| req_headers.get(header::HOST).and_then(|hdr| hdr.to_str().ok()))
            .unwrap();

        let url_path_query = req_uri.path_and_query().map(PathAndQuery::as_str).unwrap();
        format!("{}render?url={}://{}{}", service_url, scheme, host, url_path_query)
    }

    pub async fn get_rendered_response(inner: &Inner, req: ServiceRequest) -> Result<ServiceResponse, PrerenderError> {
        let mut prerender_headers = HeaderMap::new();

        // we forward every header, with the exception of `HOST`
        if inner.forward_headers {
            req.headers().iter().for_each(|a| {
                prerender_headers.append(a.0, a.1.to_str().unwrap().parse().unwrap());
            });
            prerender_headers.remove(header::HOST);
        }

        prerender_headers.insert(header::ACCEPT_ENCODING, ContentEncoding::Gzip.to_header_value());
        if let Some(token) = &inner.prerender_token {
            prerender_headers.append("X-Prerender-Token", token.parse().unwrap());
        }

        if let Some(before_render_fn) = &inner.before_render_fn {
            before_render_fn(&req, &mut prerender_headers);
        }

        let url_to_request = Self::prepare_build_api_url(&inner.prerender_service_url, &req);
        trace!("sending request to: {}", &url_to_request);

        let prerender_response = inner
            .inner_client
            .get(url_to_request)
            .headers(prerender_headers)
            .send()
            .and_then(reqwest::Response::bytes)
            .await?;

        let http_response = HttpResponse::Ok().content_type("text/html").body(prerender_response);
        Ok(req.into_response(http_response))
    }
}

impl<S, B> Service<ServiceRequest> for PrerenderMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<ServiceResponse<EitherBody<B>>, Error>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // life goes on
        if !should_prerender(&req) {
            let fut = self.service.call(req);
            return Box::pin(async move { fut.await.map(ServiceResponse::map_into_left_body) });
        }

        let inner = Rc::clone(&self.inner);
        Box::pin(async move {
            Self::get_rendered_response(&inner, req)
                .await
                .map(ServiceResponse::map_into_right_body)
                .map_err(Into::into)
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::builder::Prerender;
    use actix_web::dev::ServiceRequest;
    use actix_web::http::header;
    use actix_web::middleware::Compat;
    use actix_web::test::TestRequest;
    use actix_web::App;
    use reqwest::header::HeaderMap;
    use url::Url;

    use crate::middleware::{prerender_url, should_prerender, PrerenderMiddleware};

    fn _init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[actix_web::test]
    async fn compat_compat() {
        App::new().wrap(Compat::new(Prerender::build().use_prerender_io("".to_string())));
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

        assert!(!should_prerender(&req));
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
        Prerender::build().use_prerender_io("".to_string())
    }

    #[actix_web::test]
    async fn test_url_common() {
        let req_url = "http://yourserver.com/clothes/tshirts/red-dotted";

        let req = TestRequest::post()
            .insert_header((
                header::USER_AGENT,
                "LinkedInBot/1.0 (compatible; Mozilla/5.0; Jakarta Commons-HttpClient/3.1 +http://www.linkedin.com)",
            ))
            .uri(req_url)
            .to_srv_request();

        assert_eq!(
            PrerenderMiddleware::<()>::prepare_build_api_url(&prerender_url(), &req),
            format!("{}render?url={}", prerender_url(), req_url)
        );

        assert_eq!(
            PrerenderMiddleware::<()>::prepare_build_api_url(&Url::parse("http://localhost:5000").unwrap(), &req),
            format!("http://localhost:5000/render?url={}", req_url)
        );
    }

    #[actix_web::test]
    async fn test_url_https() {
        let req_url = "https://mercadoskin.com.br/market/csgo";

        let req = TestRequest::get()
            .insert_header((
                header::USER_AGENT,
                "LinkedInBot/1.0 (compatible; Mozilla/5.0; Jakarta Commons-HttpClient/3.1 +http://www.linkedin.com)",
            ))
            .uri(req_url)
            .to_srv_request();

        assert_eq!(
            PrerenderMiddleware::<()>::prepare_build_api_url(&Url::parse("http://localhost:5000").unwrap(), &req),
            format!("http://localhost:5000/render?url={}", req_url)
        );
    }

    #[actix_web::test]
    async fn test_url_x_forwarded_proto_single() {
        let req_url = "http://mercadoskin.com.br/market/csgo";

        let req = TestRequest::get()
            .insert_header((
                header::USER_AGENT,
                "LinkedInBot/1.0 (compatible; Mozilla/5.0; Jakarta Commons-HttpClient/3.1 +http://www.linkedin.com)",
            ))
            .insert_header(("X-Forwarded-Proto", "https"))
            .uri(req_url)
            .to_srv_request();

        assert_eq!(
            PrerenderMiddleware::<()>::prepare_build_api_url(&Url::parse("http://localhost:5000").unwrap(), &req),
            "http://localhost:5000/render?url=https://mercadoskin.com.br/market/csgo".to_string()
        );
    }

    #[actix_web::test]
    async fn test_url_x_forwarded_proto_double() {
        let req_url = "http://mercadoskin.com.br/market/csgo";

        let req = TestRequest::get()
            .insert_header((
                header::USER_AGENT,
                "LinkedInBot/1.0 (compatible; Mozilla/5.0; Jakarta Commons-HttpClient/3.1 +http://www.linkedin.com)",
            ))
            .insert_header(("X-Forwarded-Proto", "https,http"))
            .uri(req_url)
            .to_srv_request();

        assert_eq!(
            PrerenderMiddleware::<()>::prepare_build_api_url(&Url::parse("http://localhost:5000").unwrap(), &req),
            "http://localhost:5000/render?url=https://mercadoskin.com.br/market/csgo".to_string()
        );
    }

    #[actix_web::test]
    async fn test_url_cf_visitor() {
        let req_url = "http://mercadoskin.com.br/market/csgo";

        let req = TestRequest::get()
            .insert_header((
                header::USER_AGENT,
                "LinkedInBot/1.0 (compatible; Mozilla/5.0; Jakarta Commons-HttpClient/3.1 +http://www.linkedin.com)",
            ))
            .insert_header(("cf-visitor", r#""scheme":"https""#))
            .uri(req_url)
            .to_srv_request();

        assert_eq!(
            PrerenderMiddleware::<()>::prepare_build_api_url(&Url::parse("http://localhost:5000").unwrap(), &req),
            "http://localhost:5000/render?url=https://mercadoskin.com.br/market/csgo".to_string()
        );
    }

    #[actix_web::test]
    async fn test_forward_cookies() {
        let req_url = "http://mercadoskin.com.br/market/csgo";

        fn pointer_fn(_req: &ServiceRequest, _map: &mut HeaderMap) {}

        let _req = TestRequest::get()
            .insert_header((
                header::USER_AGENT,
                "LinkedInBot/1.0 (compatible; Mozilla/5.0; Jakarta Commons-HttpClient/3.1 +http://www.linkedin.com)",
            ))
            .insert_header(("cf-visitor", r#""scheme":"https""#))
            .uri(req_url)
            .to_srv_request();

        let _middleware = Prerender::build()
            .set_before_render_fn(pointer_fn)
            .use_custom_prerender_url("https://localhost:3001")
            .unwrap();
    }
}
