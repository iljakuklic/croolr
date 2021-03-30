## Croolr

A simple Rust web crawler. An experiment in asynchronous programming in Rust.

### API

* `/crawl/example.com` to crawl given domain
* `/urls/example.com` to list URLs discovered for given domain
* `/count/example.com` to count URLs discovered for given domain

### Used techniques and packages

* `async`/`await`
* Agent-style message passing
* [Tokio](https://tokio.rs) for concurrency primitives
* `warp` for server-side http handling
* `reqwest` for client-side http requests

### TODO

* Reuse HTTP connections
* Use higher-level concurrency abstractions (e.g. `tower` Service)
* Split up the big `Crawler` process/task
* Testing with `loom`
