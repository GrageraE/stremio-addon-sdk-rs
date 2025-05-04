use std::net::Ipv4Addr;
use std::env;
use stremio_addon_sdk::server::{serve_http, ServerOptions, TLSInfo};

mod manifest;
use manifest::get_manifest;

mod handlers;
use handlers::build;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // get the Manifest, which is declared in manifest.rs
    let manifest = get_manifest();

    // get the handlers, declared in handlers.rs
    let interface = build(manifest);

    let port =
        env::var("PORT")
            .ok()
            .and_then(|port| port.parse().ok())
            .unwrap_or(1337);

    // HTTP server settings
    let options = ServerOptions {
        // cache_max_age: 3600 * 24 *3, // cache for 3 days
        cache_max_age: 0,
        port,
        ip: Ipv4Addr::new(127,0,0,1).into(),
        tls: TLSInfo::NoTLS,
    };

    // run HTTP server asynchronously
    serve_http(interface, options).await
}
