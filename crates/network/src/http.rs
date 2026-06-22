use std::collections::HashMap;
use helper::Value;

fn blocking_client() -> Result<reqwest::blocking::Client, String> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))
}

fn parse_headers_arg(value: &Value) -> Result<HashMap<String, String>, String> {
    match value {
        Value::String(s) => {
            let mut headers = HashMap::new();
            for line in s.lines() {
                if let Some(pos) = line.find(':') {
                    headers.insert(
                        line[..pos].trim().to_string(),
                        line[pos + 1..].trim().to_string(),
                    );
                }
            }
            Ok(headers)
        }
        Value::Array(arr) => {
            let mut headers = HashMap::new();
            for item in arr {
                if let Value::String(s) = item {
                    if let Some(pos) = s.find(':') {
                        headers
                            .insert(s[..pos].trim().to_string(), s[pos + 1..].trim().to_string());
                    }
                }
            }
            Ok(headers)
        }
        _ => Err("Headers must be a string or array of strings".to_string()),
    }
}

fn response_to_value(resp: reqwest::blocking::Response) -> Result<Value, String> {
    let status = resp.status().as_u16() as f64;
    let headers_str = resp
        .headers()
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v.to_str().unwrap_or("")))
        .collect::<Vec<_>>()
        .join("\n");
    let body = resp
        .text()
        .map_err(|e| format!("Failed to read body: {}", e))?;

    Ok(Value::Array(vec![
        Value::Number(status),
        Value::String(headers_str),
        Value::String(body),
    ]))
}

/// Sends a request and returns a status-0 error array instead of
/// propagating network errors as runtime errors. This lets Psy code
/// handle failures gracefully with IF status == 0 THEN checks.
fn send_request(req: reqwest::blocking::RequestBuilder) -> Result<Value, String> {
    match req.send() {
        Ok(resp) => response_to_value(resp),
        Err(e) => Ok(Value::Array(vec![
            Value::Number(0.0),
            Value::String(String::new()),
            Value::String(format!("Request failed: {}", e)),
        ])),
    }
}

pub fn get(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("HTTP_GET expects at least 1 argument (url)".into());
    }
    let url = string_arg(&args[0], "url")?;
    let client = blocking_client()?;
    let mut req = client.get(&url);

    if args.len() > 1 {
        for (k, v) in parse_headers_arg(&args[1])? {
            req = req.header(k, v);
        }
    }

    send_request(req)
}

pub fn post(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("HTTP_POST expects at least 2 arguments (url, body)".into());
    }
    let url = string_arg(&args[0], "url")?;
    let body = string_arg(&args[1], "body")?;
    let client = blocking_client()?;
    let mut req = client.post(&url).body(body);

    if args.len() > 2 {
        for (k, v) in parse_headers_arg(&args[2])? {
            req = req.header(k, v);
        }
    }

    send_request(req)
}

pub fn put(args: &[Value]) -> Result<Value, String> {
    if args.len() < 2 {
        return Err("HTTP_PUT expects at least 2 arguments (url, body)".into());
    }
    let url = string_arg(&args[0], "url")?;
    let body = string_arg(&args[1], "body")?;
    let client = blocking_client()?;
    let mut req = client.put(&url).body(body);

    if args.len() > 2 {
        for (k, v) in parse_headers_arg(&args[2])? {
            req = req.header(k, v);
        }
    }

    send_request(req)
}

pub fn delete(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("HTTP_DELETE expects at least 1 argument (url)".into());
    }
    let url = string_arg(&args[0], "url")?;
    let client = blocking_client()?;
    let mut req = client.delete(&url);

    if args.len() > 1 {
        for (k, v) in parse_headers_arg(&args[1])? {
            req = req.header(k, v);
        }
    }

    send_request(req)
}

pub fn head(args: &[Value]) -> Result<Value, String> {
    if args.is_empty() {
        return Err("HTTP_HEAD expects at least 1 argument (url)".into());
    }
    let url = string_arg(&args[0], "url")?;
    let client = blocking_client()?;
    let mut req = client.head(&url);

    if args.len() > 1 {
        for (k, v) in parse_headers_arg(&args[1])? {
            req = req.header(k, v);
        }
    }

    send_request(req)
}

fn string_arg(v: &Value, name: &str) -> Result<String, String> {
    match v {
        Value::String(s) => Ok(s.clone()),
        _ => Err(format!("Expected string for '{}'", name)),
    }
}
