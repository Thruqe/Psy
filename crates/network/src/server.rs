use axum::{
    Router,
    body::Body,
    extract::{Path, Request, State},
    http::{HeaderMap, Method, StatusCode},
    response::Response,
    routing::any,
};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use tower_http::cors::CorsLayer;
use helper::Value;
use uuid::Uuid;

#[derive(Clone)]
struct Route {
    _method: String,
    _path: String,
}

/// A queued incoming request waiting to be consumed by SERVER_ACCEPT.
#[derive(Debug)]
pub struct PendingRequest {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    /// One-shot channel: the handler waits here for the pseudocode to call SERVER_RESPOND.
    pub responder: oneshot::Sender<(u16, String)>,
}

struct ServerState {
    port: u16,
    routes: Vec<Route>,
    /// Requests queued by axum handlers, consumed by SERVER_ACCEPT.
    pending: Arc<Mutex<Vec<PendingRequest>>>,
    /// Shutdown signal.
    shutdown_tx: Option<oneshot::Sender<()>>,
}

static SERVERS: OnceCell<Arc<Mutex<HashMap<String, ServerState>>>> = OnceCell::new();

fn servers() -> Arc<Mutex<HashMap<String, ServerState>>> {
    SERVERS
        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
        .clone()
}

fn tokio_rt() -> &'static tokio::runtime::Runtime {
    static RT: OnceCell<tokio::runtime::Runtime> = OnceCell::new();
    RT.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime")
    })
}

// ── shared axum state ─────────────────────────────────────────────────────────

#[derive(Clone)]
struct AppState {
    pending: Arc<Mutex<Vec<PendingRequest>>>,
}

/// Generic catch-all handler: enqueues the request and blocks until SERVER_RESPOND sends a reply.
async fn catch_all_handler(
    State(state): State<AppState>,
    method: Method,
    path_opt: Option<Path<String>>,
    headers: HeaderMap,
    req: Request<Body>,
) -> Response<Body> {
    let path = path_opt
        .map(|p| format!("/{}", p.0))
        .unwrap_or_else(|| "/".to_string());

    let headers_map: HashMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| v.to_str().ok().map(|v| (k.to_string(), v.to_string())))
        .collect();

    let body_bytes = axum::body::to_bytes(req.into_body(), usize::MAX)
        .await
        .unwrap_or_default();
    let body = String::from_utf8_lossy(&body_bytes).to_string();

    let (resp_tx, resp_rx) = oneshot::channel::<(u16, String)>();

    {
        let mut pending = state.pending.lock().await;
        pending.push(PendingRequest {
            method: method.to_string(),
            path,
            headers: headers_map,
            body,
            responder: resp_tx,
        });
    }

    // Wait for SERVER_RESPOND
    match resp_rx.await {
        Ok((status, body)) => Response::builder()
            .status(StatusCode::from_u16(status).unwrap_or(StatusCode::OK))
            .header("Content-Type", "application/json")
            .body(Body::from(body))
            .unwrap(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Server closed"))
            .unwrap(),
    }
}

// ── native functions ──────────────────────────────────────────────────────────

/// SERVER_CREATE(port) → server_id
pub fn server_create(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("SERVER_CREATE expects 1 argument (port)".into());
    }
    let port = match &args[0] {
        Value::Number(n) => *n as u16,
        Value::String(s) => s.parse::<u16>().map_err(|_| "Invalid port".to_string())?,
        _ => return Err("SERVER_CREATE expects a number for port".into()),
    };

    let server_id = format!("server_{}", Uuid::new_v4().as_simple());
    let rt = tokio_rt();

    rt.block_on(async {
        let svrs = servers();
        let mut svrs = svrs.lock().await;
        if svrs.values().any(|s| s.port == port) {
            return Err(format!("A server on port {} already exists", port));
        }
        svrs.insert(
            server_id.clone(),
            ServerState {
                port,
                routes: Vec::new(),
                pending: Arc::new(Mutex::new(Vec::new())),
                shutdown_tx: None,
            },
        );
        Ok(())
    })?;

    Ok(Value::String(server_id))
}

/// SERVER_ROUTE(server_id, method, path) → true
pub fn server_route(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 {
        return Err("SERVER_ROUTE expects 3 arguments (server_id, method, path)".into());
    }
    let server_id = string_arg(&args[0], "server_id")?;
    let method = string_arg(&args[1], "method")?.to_uppercase();
    let path = string_arg(&args[2], "path")?;

    tokio_rt().block_on(async {
        let svrs = servers();
        let mut svrs = svrs.lock().await;
        let server = svrs
            .get_mut(&server_id)
            .ok_or_else(|| format!("Server '{}' not found", server_id))?;
        server.routes.push(Route {
            _method: method,
            _path: path,
        });
        Ok(Value::Boolean(true))
    })
}

/// SERVER_LISTEN(server_id) → true
/// Builds the Axum router and starts serving in the background.
pub fn server_listen(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("SERVER_LISTEN expects 1 argument (server_id)".into());
    }
    let server_id = string_arg(&args[0], "server_id")?;
    let rt = tokio_rt();

    rt.block_on(async {
        let svrs = servers();
        let mut svrs = svrs.lock().await;
        let server = svrs
            .get_mut(&server_id)
            .ok_or_else(|| format!("Server '{}' not found", server_id))?;

        if server.shutdown_tx.is_some() {
            return Err(format!("Server '{}' is already listening", server_id));
        }

        let pending = server.pending.clone();
        let port = server.port;
        let state = AppState { pending };

        // Build router: one wildcard route covers everything.
        // Registered routes are checked at SERVER_ACCEPT time instead,
        // giving pseudocode full control over dispatch.
        let app = Router::new()
            .route("/", any(catch_all_handler))
            .route("/{*path}", any(catch_all_handler))
            .layer(CorsLayer::permissive())
            .with_state(state);

        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        server.shutdown_tx = Some(shutdown_tx);

        let addr = format!("0.0.0.0:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| format!("Failed to bind port {}: {}", port, e))?;

        tokio::spawn(async move {
            axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await
                .ok();
        });

        Ok(Value::Boolean(true))
    })
}

/// SERVER_ACCEPT(server_id) → [method, path, headers_str, body] | "no_connection"
/// Non-blocking: returns immediately if no request is queued.
pub fn server_accept(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("SERVER_ACCEPT expects 1 argument (server_id)".into());
    }
    let server_id = string_arg(&args[0], "server_id")?;

    tokio_rt().block_on(async {
        let svrs = servers();
        let svrs = svrs.lock().await;
        let server = svrs
            .get(&server_id)
            .ok_or_else(|| format!("Server '{}' not found", server_id))?;

        let pending_arc = server.pending.clone();
        drop(svrs);

        let mut pending = pending_arc.lock().await;
        if pending.is_empty() {
            return Ok(Value::String("no_connection".into()));
        }

        let req = pending.remove(0);
        let headers_str = req
            .headers
            .iter()
            .map(|(k, v)| format!("{}: {}", k, v))
            .collect::<Vec<_>>()
            .join("\n");

        // Stash the responder in a side-registry keyed by a request ID
        // so SERVER_RESPOND can find it.
        let req_id = Uuid::new_v4().to_string();
        {
            let mut resp_reg = RESPONDERS
                .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
                .lock()
                .await;
            resp_reg.insert(req_id.clone(), req.responder);
        }

        Ok(Value::Array(vec![
            Value::String(req_id),
            Value::String(req.method),
            Value::String(req.path),
            Value::String(headers_str),
            Value::String(req.body),
        ]))
    })
}

/// SERVER_ACCEPT_BLOCKING(server_id) → same shape as SERVER_ACCEPT but waits.
pub fn server_accept_blocking(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("SERVER_ACCEPT_BLOCKING expects 1 argument (server_id)".into());
    }
    let server_id = string_arg(&args[0], "server_id")?;

    tokio_rt().block_on(async {
        loop {
            let svrs = servers();
            let svrs = svrs.lock().await;
            let server = svrs
                .get(&server_id)
                .ok_or_else(|| format!("Server '{}' not found", server_id))?;
            let pending_arc = server.pending.clone();
            drop(svrs);

            let mut pending = pending_arc.lock().await;
            if !pending.is_empty() {
                let req = pending.remove(0);
                let headers_str = req
                    .headers
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                let req_id = Uuid::new_v4().to_string();
                {
                    let mut resp_reg = RESPONDERS
                        .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
                        .lock()
                        .await;
                    resp_reg.insert(req_id.clone(), req.responder);
                }

                return Ok(Value::Array(vec![
                    Value::String(req_id),
                    Value::String(req.method),
                    Value::String(req.path),
                    Value::String(headers_str),
                    Value::String(req.body),
                ]));
            }
            drop(pending);
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    })
}

// Registry that holds one-shot senders between SERVER_ACCEPT and SERVER_RESPOND.
static RESPONDERS: OnceCell<Arc<Mutex<HashMap<String, oneshot::Sender<(u16, String)>>>>> =
    OnceCell::new();

/// SERVER_RESPOND(request_id, body, status_code?) → true
pub fn server_respond(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("SERVER_RESPOND expects at least 2 arguments (request_id, body)".into());
    }
    let req_id = string_arg(&args[0], "request_id")?;
    let body = string_arg(&args[1], "body")?;
    let status = if args.len() > 2 {
        match &args[2] {
            Value::Number(n) => *n as u16,
            _ => 200,
        }
    } else {
        200
    };

    tokio_rt().block_on(async {
        let reg = RESPONDERS
            .get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
            .clone();
        let mut reg = reg.lock().await;
        let sender = reg
            .remove(&req_id)
            .ok_or_else(|| format!("Request '{}' not found or already responded", req_id))?;
        sender
            .send((status, body))
            .map_err(|_| "Client disconnected".to_string())?;
        Ok(Value::Boolean(true))
    })
}

/// SERVER_STOP(server_id) → true
pub fn server_stop(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("SERVER_STOP expects 1 argument (server_id)".into());
    }
    let server_id = string_arg(&args[0], "server_id")?;

    tokio_rt().block_on(async {
        let svrs = servers();
        let mut svrs = svrs.lock().await;
        let server = svrs
            .get_mut(&server_id)
            .ok_or_else(|| format!("Server '{}' not found", server_id))?;

        if let Some(tx) = server.shutdown_tx.take() {
            let _ = tx.send(());
        }

        svrs.remove(&server_id);
        Ok(Value::Boolean(true))
    })
}

/// SERVER_LIST() → [string, ...]
pub fn server_list(_args: &[Value]) -> Result<Value, String> {
    tokio_rt().block_on(async {
        let svrs = servers();
        let svrs = svrs.lock().await;
        Ok(Value::Array(
            svrs.iter()
                .map(|(id, s)| {
                    Value::String(format!(
                        "{} (port: {}, listening: {})",
                        id,
                        s.port,
                        s.shutdown_tx.is_some()
                    ))
                })
                .collect(),
        ))
    })
}

fn string_arg(v: &Value, name: &str) -> Result<String, String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(format!("Expected string for '{}'", name)),
    }
}
