use std::net::Ipv4Addr;
use std::env;
use stremio_addon_sdk::server::{ServerOptions, ServerOptionsWithTLSInfo, TLSInfo};

mod manifest;
use manifest::get_manifest;

mod handlers;
use handlers::build;

// Usual local deployment
#[cfg(not(feature = "shuttle"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    use stremio_addon_sdk::server::serve_http;
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
    let server_options = ServerOptions {
        // cache_max_age: 3600 * 24 *3, // cache for 3 days
        cache_max_age: 0,
        port,
        ip: Ipv4Addr::new(127,0,0,1).into(),
    };

    let options = ServerOptionsWithTLSInfo {
        server_options,
        tls_info: TLSInfo::NoTLS
    };

    // run HTTP server asynchronously
    serve_http(interface, options).await
}

// Serverless deployment in Shuttle.dev. Enable the `shuttle` feature flag to use it
#[cfg(feature = "shuttle")]
#[shuttle_runtime::main]
async fn main() -> Result<stremio_addon_sdk::server::shuttle_serverless::ServerlessShuttle, shuttle_runtime::Error> {
    use stremio_addon_sdk::server::shuttle_serverless::*;
    
    let manifest = get_manifest();
    let interface = build(manifest);

    let options = ServerOptionsWithTLSInfo {
        server_options: ServerOptions {
            cache_max_age: 0,
            port: 1337, // Ignored
            ip: Ipv4Addr::new(0, 0, 0, 0).into()    // Ignored
        },
        tls_info: TLSInfo::NoTLS    // TLS managed by Shuttle
    };

    // Start the Shuttle Service
    serve_serverless_shuttle(interface, options)
}
