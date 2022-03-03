# actix-prerender

A simple middleware that sends HTTP requests from known crawlers user-agents to
render as HTML by an external prerender service URL such as
[tvanro/prerender-alpine](https://hub.docker.com/r/tvanro/prerender-alpine
"docker hub image"), or from the service from the creators
[prerender.io](https://prerender.io "Prerender.io website").

Useful to websites with tons of javascript, such as SPAs like Vue.js or React
among others.

## Usage


### Prerender.io example
```rust

 use actix_prerender::Prerender;
 use actix_web::http::header;

 let token = "prerender service token".to_string();
 let prerender = Prerender::build().use_prerender_io(token);

 // `prerender` can now be used in `App::wrap`.
 ```

### Custom service URL example
```rust
use actix_prerender::Prerender;
use actix_web::http::header;

let token = "prerender service token".to_string();
let prerender = Prerender::build().use_custom_prerender_url("https://localhost:5001");

// `prerender` can now be used in `App::wrap`.
```

## Installation

Add this into your `Cargo.toml`

```toml
actix-prerender = "0.2"
```
