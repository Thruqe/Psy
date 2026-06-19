use std::collections::HashMap;
use std::io::Read;
use std::io::Write;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use types::Value;

// Global server registry using OnceLock
fn active_servers() -> &'static Arc<Mutex<HashMap<String, ServerState>>> {
    static SERVERS: OnceLock<Arc<Mutex<HashMap<String, ServerState>>>> = OnceLock::new();
    SERVERS.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

struct ServerState {
    listener: Option<TcpListener>,
    running: bool,
    routes: Vec<Route>,
    port: u16,
}

#[derive(Clone)]
struct Route {
    method: String,
    path: String,
    handler_id: String,
}

fn parse_http_request(request: &str) -> Result<ParsedRequest, String> {
    let mut lines = request.lines();

    // Parse request line
    let request_line = lines.next().ok_or_else(|| "Empty request".to_string())?;

    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 3 {
        return Err("Invalid request line".to_string());
    }

    let method = parts[0].to_string();
    let path = parts[1].to_string();

    // Parse headers
    let mut headers = HashMap::new();
    let mut body = String::new();
    let mut reading_body = false;

    for line in lines {
        if reading_body {
            body.push_str(line);
            body.push('\n');
        } else if line.is_empty() {
            reading_body = true;
        } else if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_lowercase();
            let value = line[pos + 1..].trim().to_string();
            headers.insert(key, value);
        }
    }

    Ok(ParsedRequest {
        method,
        path,
        headers,
        body: body.trim().to_string(),
    })
}

#[derive(Debug, Clone)]
struct ParsedRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: String,
}

fn handle_client(mut stream: TcpStream, routes: Arc<Vec<Route>>) {
    let mut buffer = [0; 8192];

    match stream.read(&mut buffer) {
        Ok(size) => {
            let request_str = String::from_utf8_lossy(&buffer[..size]).to_string();

            match parse_http_request(&request_str) {
                Ok(request) => {
                    let response = find_and_execute_route(&request, &routes);
                    let _ = stream.write_all(response.as_bytes());
                }
                Err(e) => {
                    let response = format!(
                        "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\n{}",
                        e
                    );
                    let _ = stream.write_all(response.as_bytes());
                }
            }
        }
        Err(_) => {
            let response = "HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\n\r\nError reading request";
            let _ = stream.write_all(response.as_bytes());
        }
    }
}

fn find_and_execute_route(request: &ParsedRequest, routes: &[Route]) -> String {
    for route in routes {
        if route.method == request.method || route.method == "ANY" {
            if route.path == request.path
                || request.path.starts_with(&route.path)
                || route.path == "*"
            {
                let response_body = format!(
                    "{{\n  \"method\": \"{}\",\n  \"path\": \"{}\",\n  \"headers\": {},\n  \"body\": \"{}\"\n}}",
                    request.method,
                    request.path,
                    format_headers_to_json(&request.headers),
                    request.body
                );

                return format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    response_body.len(),
                    response_body
                );
            }
        }
    }

    let body = format!(
        "{{\"error\": \"Not Found\", \"path\": \"{}\"}}",
        request.path
    );
    format!(
        "HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

fn format_headers_to_json(headers: &HashMap<String, String>) -> String {
    let mut json = String::from("{");
    let mut first = true;
    for (key, value) in headers {
        if !first {
            json.push_str(", ");
        }
        json.push_str(&format!("\"{}\": \"{}\"", key, value));
        first = false;
    }
    json.push('}');
    json
}

// Native function implementations
pub fn server_create(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("SERVER_CREATE expects at least 1 argument (port)".to_string());
    }

    let port = match &args[0] {
        Value::Number(n) => *n as u16,
        Value::String(s) => s
            .parse::<u16>()
            .map_err(|_| "Invalid port number".to_string())?,
        _ => return Err("SERVER_CREATE expects a number for port".to_string()),
    };

    let server_id = format!("server_{}", port);

    let servers = active_servers();
    let mut servers = servers.lock().map_err(|e| format!("Lock error: {}", e))?;

    if servers.contains_key(&server_id) {
        return Err(format!("Server on port {} already exists", port));
    }

    servers.insert(
        server_id.clone(),
        ServerState {
            listener: None,
            running: false,
            routes: Vec::new(),
            port,
        },
    );

    Ok(Value::String(server_id))
}

pub fn server_listen(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("SERVER_LISTEN expects 1 argument (server_id)".to_string());
    }

    let server_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("SERVER_LISTEN expects a server ID string".to_string()),
    };

    let servers = active_servers();
    let mut servers = servers.lock().map_err(|e| format!("Lock error: {}", e))?;

    let server = servers
        .get_mut(&server_id)
        .ok_or_else(|| format!("Server '{}' not found", server_id))?;

    if server.running {
        return Err(format!("Server '{}' is already running", server_id));
    }

    let port = server.port;
    let addr = format!("0.0.0.0:{}", port);
    let listener =
        TcpListener::bind(&addr).map_err(|e| format!("Failed to bind to port {}: {}", port, e))?;

    server.listener = Some(listener);
    server.running = true;

    Ok(Value::Boolean(true))
}

pub fn server_route(args: &[Value]) -> Result<Value, String> {
    if args.len() < 3 {
        return Err("SERVER_ROUTE expects 3 arguments (server_id, method, path)".to_string());
    }

    let server_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("SERVER_ROUTE expects a server ID string".to_string()),
    };

    let method = match &args[1] {
        Value::String(s) => s.to_uppercase(),
        _ => return Err("SERVER_ROUTE expects a string method".to_string()),
    };

    let path = match &args[2] {
        Value::String(s) => s.clone(),
        _ => return Err("SERVER_ROUTE expects a string path".to_string()),
    };

    let servers = active_servers();
    let mut servers = servers.lock().map_err(|e| format!("Lock error: {}", e))?;

    let server = servers
        .get_mut(&server_id)
        .ok_or_else(|| format!("Server '{}' not found", server_id))?;

    server.routes.push(Route {
        method,
        path,
        handler_id: format!("handler_{}", server.routes.len()),
    });

    Ok(Value::Boolean(true))
}

pub fn server_accept(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("SERVER_ACCEPT expects 1 argument (server_id)".to_string());
    }

    let server_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("SERVER_ACCEPT expects a server ID string".to_string()),
    };

    let servers = active_servers();
    let servers = servers.lock().map_err(|e| format!("Lock error: {}", e))?;

    let server = servers
        .get(&server_id)
        .ok_or_else(|| format!("Server '{}' not found", server_id))?;

    if !server.running {
        return Err(format!(
            "Server '{}' is not running. Call SERVER_LISTEN first.",
            server_id
        ));
    }

    let listener = server
        .listener
        .as_ref()
        .ok_or_else(|| "Server listener not initialized".to_string())?;

    // Set a timeout on the listener so it doesn't block forever
    listener
        .set_nonblocking(true)
        .map_err(|e| format!("Failed to set non-blocking: {}", e))?;

    // Try to accept a connection
    match listener.accept() {
        Ok((stream, addr)) => {
            let routes = Arc::new(server.routes.clone());
            thread::spawn(move || {
                handle_client(stream, routes);
            });
            Ok(Value::String(format!("{}", addr)))
        }
        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            // No connection waiting - return empty string to indicate no connection
            Ok(Value::String("no_connection".to_string()))
        }
        Err(e) => Err(format!("Accept error: {}", e)),
    }
}

pub fn server_accept_blocking(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("SERVER_ACCEPT_BLOCKING expects 1 argument (server_id)".to_string());
    }

    let server_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("SERVER_ACCEPT_BLOCKING expects a server ID string".to_string()),
    };

    let servers = active_servers();
    let servers = servers.lock().map_err(|e| format!("Lock error: {}", e))?;

    let server = servers
        .get(&server_id)
        .ok_or_else(|| format!("Server '{}' not found", server_id))?;

    if !server.running {
        return Err(format!(
            "Server '{}' is not running. Call SERVER_LISTEN first.",
            server_id
        ));
    }

    let listener = server
        .listener
        .as_ref()
        .ok_or_else(|| "Server listener not initialized".to_string())?;

    match listener.accept() {
        Ok((mut stream, addr)) => {
            let mut buffer = [0; 8192];
            match stream.read(&mut buffer) {
                Ok(size) => {
                    let request = String::from_utf8_lossy(&buffer[..size]).to_string();

                    let mut result = Vec::new();
                    result.push(Value::String(format!("{}", addr)));
                    result.push(Value::String(request));

                    Ok(Value::Array(result))
                }
                Err(e) => Err(format!("Read error: {}", e)),
            }
        }
        Err(e) => Err(format!("Accept error: {}", e)),
    }
}

pub fn server_respond(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("SERVER_RESPOND expects 2 arguments (server_id, response)".to_string());
    }

    let server_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("SERVER_RESPOND expects a server ID string".to_string()),
    };

    let response = match &args[1] {
        Value::String(s) => s.clone(),
        _ => return Err("SERVER_RESPOND expects a string response".to_string()),
    };

    let servers = active_servers();
    let servers = servers.lock().map_err(|e| format!("Lock error: {}", e))?;

    if !servers.contains_key(&server_id) {
        return Err(format!("Server '{}' not found", server_id));
    }

    Ok(Value::String(response))
}

pub fn server_stop(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("SERVER_STOP expects 1 argument (server_id)".to_string());
    }

    let server_id = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("SERVER_STOP expects a server ID string".to_string()),
    };

    let servers = active_servers();
    let mut servers = servers.lock().map_err(|e| format!("Lock error: {}", e))?;

    if let Some(server) = servers.get_mut(&server_id) {
        server.running = false;
        server.listener = None;
    }

    servers.remove(&server_id);
    Ok(Value::Boolean(true))
}

pub fn server_list(args: &[Value]) -> Result<Value, String> {
    let _ = args; // Unused parameter
    let servers = active_servers();
    let servers = servers.lock().map_err(|e| format!("Lock error: {}", e))?;

    let mut server_list = Vec::new();

    for (id, server) in servers.iter() {
        let info = format!(
            "{} (port: {}, running: {})",
            id, server.port, server.running
        );
        server_list.push(Value::String(info));
    }

    Ok(Value::Array(server_list))
}
