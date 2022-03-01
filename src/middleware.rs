use crate::{IGNORED_EXTENSIONS, USER_AGENTS};
use actix_web::dev::ServiceRequest;
use actix_web::http::header::HeaderMap;
use actix_web::http::uri::PathAndQuery;
use actix_web::http::{header, Method};

#[derive(Debug)]
struct PrerenderMiddleware {
    prerender_service_url: String,
}

impl PrerenderMiddleware {
    pub fn prepare_build_api_url(&self, req: &ServiceRequest) -> String {
        let req_uri = req.uri();
        let req_headers = req.headers();

        // TODO: this.host?
        let host = req
            .uri()
            .host()
            .or_else(|| {
                req_headers
                    .get("X-Forwarded-Host")
                    .and_then(|hdr| hdr.to_str().ok())
            })
            .or_else(|| {
                req_headers
                    .get(header::HOST)
                    .and_then(|hdr| hdr.to_str().ok())
            })
            .unwrap();

        let scheme = req.uri().scheme_str().unwrap_or("http");
        let url_path_query = req_uri.path_and_query().map(PathAndQuery::as_str).unwrap();

        format!(
            "{}{}://{}{}",
            &*self.prerender_service_url, scheme, host, url_path_query
        )
    }
}

#[derive(Debug)]
struct PrerenderMiddlewareBuilder {}

impl PrerenderMiddlewareBuilder {
    pub fn use_prerender_io() -> PrerenderMiddleware {
        PrerenderMiddleware {
            prerender_service_url: prerender_url().to_string(),
        }
    }

    pub fn use_custom_prerender_url(prerender_service_url: &impl ToString) -> PrerenderMiddleware {
        PrerenderMiddleware {
            prerender_service_url: prerender_service_url.to_string(),
        }
    }
}

impl PrerenderMiddleware {
    pub fn builder() -> PrerenderMiddlewareBuilder {
        PrerenderMiddlewareBuilder {}
    }
}

/// Decides if should prerender the page or not.
///
/// Will NOT prerender on the following cases:
/// * HTTP is not GET or HEAD
/// * User agent is NOT crawler
/// * Is requesting a resource on `IGNORED_EXTENSIONS`
pub fn should_prerender(req: &ServiceRequest) -> bool {
    let request_headers = req.headers();
    let mut is_crawler = false;

    if ![Method::GET, Method::HEAD].contains(req.method()) {
        return false;
    }

    let req_ua_lowercase = if let Some(user_agent) = request_headers.get(header::USER_AGENT) {
        let user_agent = user_agent.to_str();
        if let Ok(ua) = user_agent {
            ua.to_lowercase()
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
        |path_query| {
            IGNORED_EXTENSIONS
                .iter()
                .any(|ext| path_query.as_str().contains(ext))
        },
    );
    if is_ignored_extension_url {
        return false;
    }

    is_crawler
}

pub fn get_prerendered_response(req: ServiceRequest) {
    let mut prerender_request_headers = HeaderMap::new();
    let forward_headers = true;

    if forward_headers {
        prerender_request_headers = req.headers().clone();
        prerender_request_headers.remove(header::HOST);
    }

    prerender_request_headers.append(header::ACCEPT_ENCODING, "gzip".parse().unwrap());

    // TODO: accept `X-Prerender-Token`
    // prerender_request_headers.insert("X-Prerender-Token", pre_render_token);
}

pub fn prerender_url() -> &'static str {
    "https://service.prerender.io/"
}

#[cfg(test)]
mod tests {
    use crate::middleware::should_prerender;
    use actix_web::http::{header, Method};
    use actix_web::test::TestRequest;

    fn init_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[test]
    fn test_human_valid_resource() {
        let req = TestRequest::get()
            .insert_header((
                header::USER_AGENT,
                "Mozilla/5.0 (X11; Linux x86_64; rv:62.0) Gecko/20100101 Firefox/62.0",
            ))
            .uri("http://yourserver.com/clothes/tshirts?query=xl")
            .to_srv_request();

        assert!(!should_prerender(&req));
    }

    #[test]
    fn test_crawler_valid_resource() {
        let req = TestRequest::get()
            .insert_header((
                header::USER_AGENT,
                "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)",
            ))
            .uri("http://yourserver.com/clothes/tshirts?query=xl")
            .to_srv_request();

        assert!(should_prerender(&req));
    }

    #[test]
    fn test_crawler_ignored_resource() {
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

    #[test]
    fn test_crawler_wrong_http_method() {
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
}
