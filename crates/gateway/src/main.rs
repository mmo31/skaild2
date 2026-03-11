use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Deserialize)]
struct GatewayRoute {
    #[allow(dead_code)]
    id: String,
    host: String,
    path_prefix: String,
    access_mode: String,
    upstream_url: String,
}

type RoutesCache = Arc<RwLock<Vec<GatewayRoute>>>;

#[derive(Clone)]
struct AppState {
    routes: RoutesCache,
    client: Client,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();

    let control_plane_url = std::env::var("SKAILD2_CONTROL_PLANE_URL")
        .unwrap_or_else(|_| "http://control-plane:8080".to_string());

    let client = Client::new();
    let routes: RoutesCache = Arc::new(RwLock::new(vec![]));

    // Initial route load
    load_routes(&client, &control_plane_url, &routes).await;

    // Background refresh every 30 seconds
    let bg_client = client.clone();
    let bg_url = control_plane_url.clone();
    let bg_routes = routes.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        interval.tick().await; // skip first tick (already loaded above)
        loop {
            interval.tick().await;
            load_routes(&bg_client, &bg_url, &bg_routes).await;
        }
    });

    let state = AppState { routes, client };

    let app = Router::new()
        .route("/health", axum::routing::get(health_check))
        .fallback(any(proxy_handler))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 80));
    tracing::info!("Gateway listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}

async fn load_routes(client: &Client, base_url: &str, cache: &RoutesCache) {
    let url = format!("{}/api/internal/routes", base_url);
    match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.json::<Vec<GatewayRoute>>().await {
            Ok(routes) => {
                tracing::info!("Loaded {} routes from control-plane", routes.len());
                *cache.write().await = routes;
            }
            Err(e) => tracing::warn!("Failed to parse routes response: {}", e),
        },
        Ok(resp) => tracing::warn!("Control-plane returned {} for routes", resp.status()),
        Err(e) => tracing::warn!("Failed to fetch routes from control-plane: {}", e),
    }
}

fn match_route<'a>(routes: &'a [GatewayRoute], host: &str, path: &str) -> Option<&'a GatewayRoute> {
    // Routes are pre-sorted longest path_prefix first by the control-plane query
    routes.iter().find(|r| r.host == host && path.starts_with(&r.path_prefix))
}

async fn proxy_handler(State(state): State<AppState>, req: Request) -> Response {
    let host = req
        .headers()
        .get("host")
        .and_then(|v| v.to_str().ok())
        .map(|h| h.split(':').next().unwrap_or(h).to_string())
        .unwrap_or_default();

    let path = req.uri().path().to_string();
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str().to_string())
        .unwrap_or_else(|| "/".to_string());

    tracing::debug!("Incoming request: host={} path={}", host, path);

    let routes = state.routes.read().await;
    let matched = match_route(&routes, &host, &path);

    let Some(route) = matched else {
        tracing::warn!("No route matched: host={} path={}", host, path);
        return (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({"error": "No route found"})),
        )
            .into_response();
    };

    match route.access_mode.as_str() {
        "login_required" => {
            // TODO(Epic 3): verify OIDC session cookie; redirect to IdP when absent
            (
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({"error": "Authentication required"})),
            )
                .into_response()
        }
        "public" => {
            let target_url = format!(
                "{}{}",
                route.upstream_url.trim_end_matches('/'),
                path_and_query
            );
            tracing::info!("Proxying {} {} -> {}", host, path, target_url);
            drop(routes); // release read lock before async I/O
            forward_request(state.client, req, &target_url).await
        }
        _ => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({"error": "Invalid route access mode"})),
        )
            .into_response(),
    }
}

async fn forward_request(client: Client, req: Request, target_url: &str) -> Response {
    let method = reqwest::Method::from_bytes(req.method().as_str().as_bytes())
        .unwrap_or(reqwest::Method::GET);
    let headers = req.headers().clone();
    let body_bytes = to_bytes(req.into_body(), 64 * 1024 * 1024)
        .await
        .unwrap_or_default();

    let mut request_builder = client.request(method, target_url).body(body_bytes.to_vec());

    for (name, value) in &headers {
        // Skip hop-by-hop headers
        if matches!(
            name.as_str(),
            "host" | "connection" | "transfer-encoding" | "te" | "trailer" | "upgrade"
        ) {
            continue;
        }
        request_builder = request_builder.header(name.clone(), value.clone());
    }

    match request_builder.send().await {
        Ok(upstream_resp) => {
            let status = StatusCode::from_u16(upstream_resp.status().as_u16())
                .unwrap_or(StatusCode::BAD_GATEWAY);
            let resp_headers = upstream_resp.headers().clone();
            let resp_body = upstream_resp.bytes().await.unwrap_or_default();

            let mut response = Response::new(Body::from(resp_body));
            *response.status_mut() = status;
            for (name, value) in &resp_headers {
                if matches!(name.as_str(), "connection" | "transfer-encoding") {
                    continue;
                }
                response.headers_mut().insert(name.clone(), value.clone());
            }
            response
        }
        Err(e) => {
            tracing::error!("Upstream proxy error: {}", e);
            (
                StatusCode::BAD_GATEWAY,
                axum::Json(serde_json::json!({"error": "Bad gateway"})),
            )
                .into_response()
        }
    }
}
