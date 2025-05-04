use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use hyper::{Request, Response, StatusCode};
use hyper::service::service_fn;
use hyper::body::Bytes;
use http_body_util::Full;
use flexible_hyper_server_tls::{HttpOrHttpsAcceptor, rustls_helpers};
use tokio::net::TcpListener;

use super::router::Router;
use super::builder::BuilderWithHandlers;

/// Contains info for the HTTPS feature
#[derive(Debug, Clone)]
pub enum TLSInfo {
    NoTLS,
    TLS {
        cert_path: String,
        key_path: String
    }
}

impl Default for TLSInfo {
    /// Default: No TLS
    fn default() -> Self {
        Self::NoTLS
    }
}

#[derive(Debug, Clone, Default)]
pub struct ServerOptionsWithTLSInfo {
    pub server_options: ServerOptions,
    pub tls_info: TLSInfo
}

#[derive(Debug, Clone, Copy)]
pub struct ServerOptions {
    pub port: u16,
    /// In seconds
    pub cache_max_age: i32,
    pub ip: IpAddr,
}

impl Default for ServerOptions {
    /// The default is: cache_max_age = 3 days, port = 7070, ip = 127.0.0.1, no TLS
    fn default() -> Self {
        Self {
            // cache 3 days
            cache_max_age: 24 * 3600 * 3,
            port: 7070,
            ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
        }
    }
}

/// Start the HTTP server
pub async fn serve_http(build: BuilderWithHandlers, options: ServerOptionsWithTLSInfo) 
    -> Result<(), Box<dyn std::error::Error>> {
    
    let (options, tls_info) = (options.server_options, options.tls_info);
    let addr = SocketAddr::new(options.ip, options.port);
    
    let listener = TcpListener::bind(addr).await?;
    let mut listener_tls = HttpOrHttpsAcceptor::new(listener);

    if let TLSInfo::TLS { cert_path, key_path } = tls_info {
        let tls = rustls_helpers::get_tlsacceptor_from_files(cert_path, key_path).await?;
        listener_tls = listener_tls.with_tls(tls);
    }

    let router = Arc::new(Router::new(build, options));
    println!("Running on: {}", addr);
    
    loop {
        let router_ptr = Arc::clone(&router);

        let service = service_fn(move |req: Request<hyper::body::Incoming>| {
            let router_ptr = Arc::clone(&router_ptr);
            async move {
                match router_ptr.route(req).await {
                    Ok(router_response) => Ok::<Response<Full<Bytes>>, Infallible>(router_response.response()),
                    Err(error) => {
                        eprintln!("service error: {:?}", error);
                        let mut response 
                            = Response::new(Full::new(Bytes::new()));
                        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                        Ok(response)
                    }
                }
            }
        });

        // let (stream, _) = listener_tls.accept(service).await.expect("Can listen");
        if let Ok((_peer_addr, conn_fut)) = listener_tls.accept(service).await {
            tokio::task::spawn(async move {
                if let Err(err) = conn_fut.await {
                    eprintln!("Error serving connection: {:?}", err);
                }
            });    
        }
    }
}

// pub fn serve_serverless(
//     req: now_lambda::Request, build: BuilderWithHandlers, options: ServerOptions
// ) -> Result<impl now_lambda::IntoResponse, now_lambda::error::NowError> {
//     let router = Router::new(build, options);
//     match router.route(req) {
//         Ok(router_response) => Ok(router_response),
//         Err(error) => {
//             let error_message = format!("service error: {:?}", error);
//             eprintln!("{}", error_message);
//             Err(now_lambda::error::NowError::new(&error_message))
//         }
//     }
// }
