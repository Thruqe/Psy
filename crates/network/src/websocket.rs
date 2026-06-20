use futures_util::{SinkExt, StreamExt};
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async, tungstenite::protocol::Message,
};
use types::Value;
use uuid::Uuid;

type WsSink = futures_util::stream::SplitSink<
    WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>,
    Message,
>;
type WsStream =
    futures_util::stream::SplitStream<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>;

struct WsConn {
    sink: WsSink,
    stream: WsStream,
}

static CONNS: OnceCell<Arc<Mutex<HashMap<String, WsConn>>>> = OnceCell::new();

fn conns() -> Arc<Mutex<HashMap<String, WsConn>>> {
    CONNS
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

/// WS_CONNECT(url) → connection_id
pub fn connect(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("WS_CONNECT expects 1 argument (url)".into());
    }
    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("WS_CONNECT expects a string URL".into()),
    };

    if !url.starts_with("ws://") && !url.starts_with("wss://") {
        return Err("WebSocket URL must start with ws:// or wss://".into());
    }

    tokio_rt().block_on(async {
        let (ws_stream, _) = connect_async(&url)
            .await
            .map_err(|e| format!("WS connect failed: {}", e))?;

        let (sink, stream) = ws_stream.split();
        let conn_id = Uuid::new_v4().to_string();

        let registry = conns();
        let mut registry = registry.lock().await;
        registry.insert(conn_id.clone(), WsConn { sink, stream });

        Ok(Value::String(conn_id))
    })
}

/// WS_SEND(connection_id, message) → true
pub fn send(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("WS_SEND expects 2 arguments (connection_id, message)".into());
    }
    let conn_id = string_arg(&args[0], "connection_id")?;
    let message = string_arg(&args[1], "message")?;

    tokio_rt().block_on(async {
        let registry = conns();
        let mut registry = registry.lock().await;
        let conn = registry
            .get_mut(&conn_id)
            .ok_or_else(|| format!("Connection '{}' not found", conn_id))?;

        conn.sink
            .send(Message::Text(message.into()))
            .await
            .map_err(|e| format!("WS send failed: {}", e))?;

        Ok(Value::Boolean(true))
    })
}

/// WS_RECEIVE(connection_id) → string message | Undefined (if closed)
pub fn receive(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("WS_RECEIVE expects 1 argument (connection_id)".into());
    }
    let conn_id = string_arg(&args[0], "connection_id")?;

    tokio_rt().block_on(async {
        let registry = conns();
        let mut registry = registry.lock().await;
        let conn = registry
            .get_mut(&conn_id)
            .ok_or_else(|| format!("Connection '{}' not found", conn_id))?;

        match conn.stream.next().await {
            Some(Ok(Message::Text(txt))) => Ok(Value::String(txt.to_string())),
            Some(Ok(Message::Binary(bin))) => {
                Ok(Value::String(String::from_utf8_lossy(&bin).to_string()))
            }
            Some(Ok(Message::Ping(_))) | Some(Ok(Message::Pong(_))) => {
                Ok(Value::String("".to_string()))
            }
            Some(Ok(Message::Close(_))) | None => Ok(Value::Undefined),
            Some(Err(e)) => Err(format!("WS receive error: {}", e)),
            Some(Ok(_)) => Ok(Value::Undefined),
        }
    })
}

/// WS_CLOSE(connection_id) → true
pub fn close(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("WS_CLOSE expects 1 argument (connection_id)".into());
    }
    let conn_id = string_arg(&args[0], "connection_id")?;

    tokio_rt().block_on(async {
        let registry = conns();
        let mut registry = registry.lock().await;
        if let Some(mut conn) = registry.remove(&conn_id) {
            conn.sink
                .send(Message::Close(None))
                .await
                .map_err(|e| format!("WS close failed: {}", e))?;
        }
        Ok(Value::Boolean(true))
    })
}

fn string_arg(v: &Value, name: &str) -> Result<String, String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(format!("Expected string for '{}'", name)),
    }
}
