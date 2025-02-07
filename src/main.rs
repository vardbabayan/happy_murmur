use axum::{
    extract::{ConnectInfo, State},
    http::Request,
    middleware::{self, Next},
    response::IntoResponse,
    routing::get,
    Router,
};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let ip_counts: Arc<Mutex<HashMap<IpAddr, usize>>> = Arc::new(Mutex::new(HashMap::new()));

    {
        let ip_counts = Arc::clone(&ip_counts);
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                
                let counts = ip_counts.lock().await;
                let mut sorted: Vec<_> = counts.iter().collect();

                // Sort IPs by count in descending order.
                sorted.sort_by(|a, b| b.1.cmp(a.1));
                
                println!("IPs:");
                for (ip, count) in sorted {
                    println!("{}: {}", ip, count);
                }
                println!("-------------------------");
            }
        });
    }

    let app = Router::new()
        .route("/ping", get(ping_handler))
        .route_layer(middleware::from_fn_with_state(
            Arc::clone(&ip_counts),
            count_middleware,
        ));

    let addr = SocketAddr::from(([127, 4, 4, 4], 4444));
    println!("Listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}

/// Handler for /ping route
async fn ping_handler() -> impl IntoResponse {
    "hi murmur"
}

/// Middleware that counts requests and adds them to the IP counter
async fn count_middleware<B>(
    State(ip_counts): State<Arc<Mutex<HashMap<IpAddr, usize>>>>,
    req: Request<B>,
    next: Next<B>,
) -> impl IntoResponse {
    if let Some(ConnectInfo(socket_addr)) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
        let ip = socket_addr.ip();
        let mut counts = ip_counts.lock().await;
        *counts.entry(ip).or_insert(0) += 1;
    }

    next.run(req).await
}
