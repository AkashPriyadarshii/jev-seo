use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::mcp::{handle_request, RpcRequest, RpcResponse};

fn http_response(stream: &mut TcpStream, status: &str, ctype: &str, body: &str) {
    let resp = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: Content-Type\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nConnection: close\r\n\r\n{}",
        status,
        ctype,
        body.len(),
        body
    );
    let _ = stream.write_all(resp.as_bytes());
}

fn handle_stream(mut stream: TcpStream) {
    let mut buf = vec![0u8; 64 * 1024];
    let n = match stream.read(&mut buf) {
        Ok(0) | Err(_) => return,
        Ok(n) => n,
    };
    let req = String::from_utf8_lossy(&buf[..n]);

    // CORS preflight
    if req.starts_with("OPTIONS ") {
        http_response(&mut stream, "204 No Content", "text/plain", "");
        return;
    }

    // Health / manifest
    if req.contains("GET /health")
        || req.contains("GET /plugin.json")
        || req.contains("GET /.well-known")
    {
        let path = if req.contains("/plugin.json") {
            "plugin.json"
        } else {
            ".well-known/plugin.json"
        };
        // Try plugin.json, else minimal manifest
        let body = std::fs::read_to_string("plugin.json")
            .or_else(|_| std::fs::read_to_string(path))
            .unwrap_or_else(|_| r#"{"name":"jev-seo"}"#.into());
        http_response(&mut stream, "200 OK", "application/json", &body);
        return;
    }

    // Serve extensions static (sidebar/panel/viewer)
    if req.starts_with("GET /extensions/") {
        let line = req.lines().next().unwrap_or("");
        let p = line
            .split(' ')
            .nth(1)
            .unwrap_or("/")
            .split('?')
            .next()
            .unwrap_or("/");
        let safe = p.trim_start_matches('/').replace("..", "");
        let body = std::fs::read_to_string(&safe).unwrap_or_else(|_| "<h1>not found</h1>".into());
        let ctype = if safe.ends_with(".html") {
            "text/html"
        } else {
            "application/json"
        };
        http_response(&mut stream, "200 OK", ctype, &body);
        return;
    }

    // POST /mcp — JSON-RPC
    if req.contains("POST /mcp") || req.contains("POST / ") {
        let body_start = req.find("\r\n\r\n").map(|i| i + 4).unwrap_or(req.len());
        let body = &req[body_start..];
        // Bodies may be truncated by single read; try to read Content-Length remainder
        let parsed: serde_json::Value =
            serde_json::from_str(body.trim()).unwrap_or(serde_json::Value::Null);
        let reply: String = if parsed.is_null() {
            serde_json::to_string(&RpcResponse {
                jsonrpc: "2.0".into(),
                id: None,
                result: None,
                error: Some(serde_json::json!({"code": -32700, "message": "Parse error"})),
            })
            .unwrap()
        } else if let Some(arr) = parsed.as_array() {
            // batch
            let out: Vec<RpcResponse> = arr
                .iter()
                .filter_map(|v| serde_json::from_value::<RpcRequest>(v.clone()).ok())
                .map(|r| handle_request(&r))
                .collect();
            serde_json::to_string(&out).unwrap()
        } else {
            match serde_json::from_value::<RpcRequest>(parsed) {
                Ok(r) => serde_json::to_string(&handle_request(&r)).unwrap(),
                Err(_) => serde_json::to_string(&RpcResponse {
                    jsonrpc: "2.0".into(),
                    id: None,
                    result: None,
                    error: Some(serde_json::json!({"code": -32700, "message": "Parse error"})),
                })
                .unwrap(),
            }
        };
        http_response(&mut stream, "200 OK", "application/json", &reply);
        return;
    }

    http_response(&mut stream, "404 Not Found", "text/plain", "not found");
}

/// Run HTTP bridge on addr (e.g. "0.0.0.0:8787"). Each connection handled on own thread.
/// ponytail: single-thread accept loop, per-connection threads; pool if >100 rps.
pub fn run_plugin_bridge(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr)?;
    eprintln!("jev-seo plugin bridge listening on http://{addr}  POST /mcp");
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                thread::spawn(move || handle_stream(s));
            }
            Err(e) => eprintln!("accept error: {e}"),
        }
    }
    Ok(())
}
