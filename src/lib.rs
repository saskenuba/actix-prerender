//! # Example
//! ```no_run
//! use actix_prerender::Prerender;
//! use actix_web::{get, http, web, App, HttpRequest, HttpResponse, HttpServer};
//!
//! #[get("/index.html")]
//! async fn index(req: HttpRequest) -> &'static str {
//!     "<p>Hello World!</p>"
//! }
//!
//! #[actix_web::main]
//! async fn main() -> std::io::Result<()> {
//!     HttpServer::new(|| {
//!         let prerender = Prerender::build().use_prerender_io("service_token".to_string());
//!
//!         App::new()
//!             .wrap(prerender)
//!             .service(index)
//!     })
//!     .bind(("127.0.0.1", 8080))?
//!     .run()
//!     .await;
//!
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![deny(nonstandard_style)]
#![allow(clippy::must_use_candidate, clippy::missing_panics_doc, clippy::missing_errors_doc)]
#![warn(future_incompatible)]
#![doc(html_logo_url = "https://actix.rs/img/logo.png")]
#![doc(html_favicon_url = "https://actix.rs/favicon.ico")]

use consts::{IGNORED_EXTENSIONS, USER_AGENTS};

mod builder;
mod consts;
mod error;
mod middleware;

pub use builder::Prerender;
pub use error::PrerenderError;
pub use middleware::PrerenderMiddleware;
