use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use types::Value;

fn parse_url(url: &str) -> Result<(String, String, u16, String), String> {
    // Parse URL into components
    let without_protocol = if let Some(stripped) = url.strip_prefix("http://") {
        (false, stripped.to_string())
    } else if let Some(stripped) = url.strip_prefix("https://") {
        (true, stripped.to_string())
    } else {
        // Default to HTTP if no protocol specified
        (false, url.to_string())
    };

    let (https, rest) = without_protocol;
    let default_port = if https { 443 } else { 80 };

    // Split host and path
    let (host_part, path) = if let Some(pos) = rest.find('/') {
        (rest[..pos].to_string(), rest[pos..].to_string())
    } else {
        (rest.clone(), "/".to_string())
    };

    // Split host and port
    let (host, port) = if let Some(pos) = host_part.find(':') {
        let port_str = &host_part[pos + 1..];
        let port = port_str
            .parse::<u16>()
            .map_err(|_| format!("Invalid port: {}", port_str))?;
        (host_part[..pos].to_string(), port)
    } else {
        (host_part, default_port)
    };

    Ok((
        host,
        path,
        port,
        if https { "https" } else { "http" }.to_string(),
    ))
}

fn build_http_request(
    method: &str,
    host: &str,
    path: &str,
    headers: &HashMap<String, String>,
    body: Option<&str>,
) -> String {
    let mut request = format!("{} {} HTTP/1.1\r\n", method, path);
    request.push_str(&format!("Host: {}\r\n", host));
    request.push_str("User-Agent: Psy-Network/1.0\r\n");
    request.push_str("Accept: */*\r\n");
    request.push_str("Connection: close\r\n");

    for (key, value) in headers {
        request.push_str(&format!("{}: {}\r\n", key, value));
    }

    if let Some(body_content) = body {
        request.push_str(&format!("Content-Length: {}\r\n", body_content.len()));
        request.push_str("Content-Type: application/json\r\n");
        request.push_str("\r\n");
        request.push_str(body_content);
    } else {
        request.push_str("\r\n");
    }

    request
}

fn parse_http_response(response: &str) -> Result<Value, String> {
    let mut lines = response.lines();

    // Parse status line
    let status_line = lines.next().ok_or_else(|| "Empty response".to_string())?;

    let parts: Vec<&str> = status_line.split_whitespace().collect();
    if parts.len() < 3 {
        return Err(format!("Invalid status line: {}", status_line));
    }

    let status_code = parts[1]
        .parse::<f64>()
        .map_err(|_| format!("Invalid status code: {}", parts[1]))?;

    // Parse headers
    let mut headers = HashMap::new();

    for line in &mut lines {
        if line.is_empty() {
            break;
        }
        if let Some(pos) = line.find(':') {
            let key = line[..pos].trim().to_lowercase();
            let value = line[pos + 1..].trim().to_string();
            headers.insert(key, value);
        }
    }

    // Collect body
    let body: String = lines.collect::<Vec<&str>>().join("\n");

    // Build response object
    let mut response_array = Vec::new();

    // Status code
    response_array.push(Value::Number(status_code));

    // Headers as string representation
    let headers_str = headers
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<String>>()
        .join("\n");
    response_array.push(Value::String(headers_str));

    // Body
    response_array.push(Value::String(body));

    Ok(Value::Array(response_array))
}

fn make_http_request(
    method: &str,
    url: &str,
    headers: Option<HashMap<String, String>>,
    body: Option<&str>,
) -> Result<Value, String> {
    let (host, path, port, protocol) = match parse_url(url) {
        Ok(result) => result,
        Err(e) => {
            // Return error response array
            let mut error_response = Vec::new();
            error_response.push(Value::Number(0.0)); // Status code 0 = error
            error_response.push(Value::String(String::new())); // Empty headers
            error_response.push(Value::String(format!("Error: {}", e))); // Error message in body
            return Ok(Value::Array(error_response));
        }
    };

    if protocol == "https" {
        let mut error_response = Vec::new();
        error_response.push(Value::Number(0.0));
        error_response.push(Value::String(String::new()));
        error_response.push(Value::String(
            "HTTPS is not supported in this version. Use HTTP or implement TLS.".to_string(),
        ));
        return Ok(Value::Array(error_response));
    }

    let headers = headers.unwrap_or_default();
    let request = build_http_request(method, &host, &path, &headers, body);

    let addr = format!("{}:{}", host, port);
    let timeout = Duration::from_secs(30);

    let socket_addrs = match addr.to_socket_addrs() {
        Ok(addrs) => addrs,
        Err(e) => {
            let mut error_response = Vec::new();
            error_response.push(Value::Number(0.0));
            error_response.push(Value::String(String::new()));
            error_response.push(Value::String(format!(
                "Failed to resolve address '{}': {}",
                addr, e
            )));
            return Ok(Value::Array(error_response));
        }
    };

    let mut stream = None;
    let mut last_error = String::new();

    for socket_addr in socket_addrs {
        match TcpStream::connect_timeout(&socket_addr, timeout) {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(e) => {
                last_error = format!("{}", e);
            }
        }
    }

    let mut stream = match stream {
        Some(s) => s,
        None => {
            let mut error_response = Vec::new();
            error_response.push(Value::Number(0.0));
            error_response.push(Value::String(String::new()));
            error_response.push(Value::String(format!(
                "Connection failed to '{}': {}",
                addr, last_error
            )));
            return Ok(Value::Array(error_response));
        }
    };

    if let Err(e) = stream.set_read_timeout(Some(timeout)) {
        let mut error_response = Vec::new();
        error_response.push(Value::Number(0.0));
        error_response.push(Value::String(String::new()));
        error_response.push(Value::String(format!("Failed to set timeout: {}", e)));
        return Ok(Value::Array(error_response));
    }

    if let Err(e) = stream.write_all(request.as_bytes()) {
        let mut error_response = Vec::new();
        error_response.push(Value::Number(0.0));
        error_response.push(Value::String(String::new()));
        error_response.push(Value::String(format!("Failed to send request: {}", e)));
        return Ok(Value::Array(error_response));
    }

    let mut response = String::new();
    if let Err(e) = stream.read_to_string(&mut response) {
        let mut error_response = Vec::new();
        error_response.push(Value::Number(0.0));
        error_response.push(Value::String(String::new()));
        error_response.push(Value::String(format!("Failed to read response: {}", e)));
        return Ok(Value::Array(error_response));
    }

    match parse_http_response(&response) {
        Ok(result) => Ok(result),
        Err(e) => {
            let mut error_response = Vec::new();
            error_response.push(Value::Number(0.0));
            error_response.push(Value::String(String::new()));
            error_response.push(Value::String(format!("Failed to parse response: {}", e)));
            Ok(Value::Array(error_response))
        }
    }
}

pub fn get(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("HTTP_GET expects at least 1 argument (url)".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("HTTP_GET expects a string URL".to_string()),
    };

    let headers = if args.len() > 1 {
        Some(parse_headers_arg(&args[1])?)
    } else {
        None
    };

    make_http_request("GET", &url, headers, None)
}

pub fn post(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("HTTP_POST expects at least 2 arguments (url, body)".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("HTTP_POST expects a string URL".to_string()),
    };

    let body = match &args[1] {
        Value::String(s) => s.clone(),
        _ => return Err("HTTP_POST expects a string body".to_string()),
    };

    let headers = if args.len() > 2 {
        Some(parse_headers_arg(&args[2])?)
    } else {
        None
    };

    make_http_request("POST", &url, headers, Some(&body))
}

pub fn put(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("HTTP_PUT expects at least 2 arguments (url, body)".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("HTTP_PUT expects a string URL".to_string()),
    };

    let body = match &args[1] {
        Value::String(s) => s.clone(),
        _ => return Err("HTTP_PUT expects a string body".to_string()),
    };

    let headers = if args.len() > 2 {
        Some(parse_headers_arg(&args[2])?)
    } else {
        None
    };

    make_http_request("PUT", &url, headers, Some(&body))
}

pub fn delete(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("HTTP_DELETE expects at least 1 argument (url)".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("HTTP_DELETE expects a string URL".to_string()),
    };

    let headers = if args.len() > 1 {
        Some(parse_headers_arg(&args[1])?)
    } else {
        None
    };

    make_http_request("DELETE", &url, headers, None)
}

pub fn head(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("HTTP_HEAD expects at least 1 argument (url)".to_string());
    }

    let url = match &args[0] {
        Value::String(s) => s.clone(),
        _ => return Err("HTTP_HEAD expects a string URL".to_string()),
    };

    let headers = if args.len() > 1 {
        Some(parse_headers_arg(&args[1])?)
    } else {
        None
    };

    make_http_request("HEAD", &url, headers, None)
}

fn parse_headers_arg(value: &Value) -> Result<HashMap<String, String>, String> {
    match value {
        Value::String(s) => {
            let mut headers = HashMap::new();
            for line in s.lines() {
                if let Some(pos) = line.find(':') {
                    let key = line[..pos].trim().to_string();
                    let value = line[pos + 1..].trim().to_string();
                    headers.insert(key, value);
                }
            }
            Ok(headers)
        }
        Value::Array(arr) => {
            let mut headers = HashMap::new();
            for item in arr {
                if let Value::String(s) = item {
                    if let Some(pos) = s.find(':') {
                        let key = s[..pos].trim().to_string();
                        let value = s[pos + 1..].trim().to_string();
                        headers.insert(key, value);
                    }
                }
            }
            Ok(headers)
        }
        _ => Err("Headers must be a string or array of strings".to_string()),
    }
}
