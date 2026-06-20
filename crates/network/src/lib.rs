pub mod fetch;
pub mod http;
pub mod server;
pub mod websocket;

use types::Value;

pub fn http_get(args: &[Value]) -> Result<Value, String> {
    http::get(args)
}
pub fn http_post(args: &[Value]) -> Result<Value, String> {
    http::post(args)
}
pub fn http_put(args: &[Value]) -> Result<Value, String> {
    http::put(args)
}
pub fn http_delete(args: &[Value]) -> Result<Value, String> {
    http::delete(args)
}
pub fn http_head(args: &[Value]) -> Result<Value, String> {
    http::head(args)
}

pub fn fetch_url(args: &[Value]) -> Result<Value, String> {
    fetch::fetch_url(args)
}
pub fn url_encode(args: &[Value]) -> Result<Value, String> {
    fetch::url_encode(args)
}
pub fn url_decode(args: &[Value]) -> Result<Value, String> {
    fetch::url_decode(args)
}
pub fn parse_json(args: &[Value]) -> Result<Value, String> {
    fetch::parse_json(args)
}

pub fn server_create(args: &[Value]) -> Result<Value, String> {
    server::server_create(args)
}
pub fn server_listen(args: &[Value]) -> Result<Value, String> {
    server::server_listen(args)
}
pub fn server_route(args: &[Value]) -> Result<Value, String> {
    server::server_route(args)
}
pub fn server_accept(args: &[Value]) -> Result<Value, String> {
    server::server_accept(args)
}
pub fn server_accept_blocking(args: &[Value]) -> Result<Value, String> {
    server::server_accept_blocking(args)
}
pub fn server_respond(args: &[Value]) -> Result<Value, String> {
    server::server_respond(args)
}
pub fn server_stop(args: &[Value]) -> Result<Value, String> {
    server::server_stop(args)
}
pub fn server_list(args: &[Value]) -> Result<Value, String> {
    server::server_list(args)
}

pub fn websocket_connect(args: &[Value]) -> Result<Value, String> {
    websocket::connect(args)
}
pub fn websocket_send(args: &[Value]) -> Result<Value, String> {
    websocket::send(args)
}
pub fn websocket_receive(args: &[Value]) -> Result<Value, String> {
    websocket::receive(args)
}
pub fn websocket_close(args: &[Value]) -> Result<Value, String> {
    websocket::close(args)
}
