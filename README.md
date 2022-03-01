# actix-prerender

A very simple middleware that sends requests which comes from common crawler
user-agents to be pre-rendered via "prerender".

It accepts the external service provided by `prerender.io`, or a custom external
`prerender_service_url`.
