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

fn header_len(head: &str, name: &str) -> Option<usize> {
    for line in head.lines().skip(1) {
        let line = line.trim_end_matches('\r');
        if line.is_empty() {
            break;
        }
        if let Some((k, v)) = line.split_once(':') {
            if k.trim().eq_ignore_ascii_case(name) {
                return v.trim().parse::<usize>().ok();
            }
        }
    }
    None
}

fn read_request(stream: &mut TcpStream) -> Option<Vec<u8>> {
    // Headers first, then the body per Content-Length. A single read()
    // is not enough: proxies split head and body across TCP segments.
    let mut raw: Vec<u8> = Vec::with_capacity(8 * 1024);
    let mut tmp = [0u8; 8 * 1024];
    let head_end = loop {
        let n = stream.read(&mut tmp).ok()?;
        if n == 0 {
            return None;
        }
        raw.extend_from_slice(&tmp[..n]);
        if let Some(i) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
            break i + 4;
        }
        if raw.len() > 64 * 1024 {
            return None;
        }
    };
    let head = String::from_utf8_lossy(&raw[..head_end]).into_owned();
    let mut body = raw[head_end..].to_vec();
    if let Some(clen) = header_len(&head, "content-length") {
        while body.len() < clen {
            let n = stream.read(&mut tmp).ok()?;
            if n == 0 {
                break;
            }
            body.extend_from_slice(&tmp[..n]);
            if body.len() > 4 * 1024 * 1024 {
                break;
            }
        }
        body.truncate(clen);
    }
    let mut full = head.into_bytes();
    full.extend_from_slice(&body);
    Some(full)
}

fn handle_stream(mut stream: TcpStream) {
    let raw = match read_request(&mut stream) {
        Some(r) => r,
        None => return,
    };
    let req = String::from_utf8_lossy(&raw);

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
