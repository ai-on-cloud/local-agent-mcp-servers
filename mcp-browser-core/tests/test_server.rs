//! Embedded HTTP server for integration tests.
//!
//! Serves bundled HTML test pages on a random port so tests don't depend on
//! external sites like example.com or httpbin.org.

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

const SIMPLE_HTML: &str = include_str!("pages/simple.html");
const FORM_HTML: &str = include_str!("pages/form.html");
const TABLE_HTML: &str = include_str!("pages/table.html");
const DYNAMIC_HTML: &str = include_str!("pages/dynamic.html");

pub struct TestServer {
    pub base_url: String,
    _shutdown: oneshot::Sender<()>,
}

impl TestServer {
    pub async fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind to random port");
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("http://127.0.0.1:{}", port);

        let (tx, rx) = oneshot::channel::<()>();

        tokio::spawn(async move {
            tokio::select! {
                _ = Self::serve_loop(listener) => {}
                _ = rx => {}
            }
        });

        TestServer {
            base_url,
            _shutdown: tx,
        }
    }

    pub fn url(&self, page: &str) -> String {
        format!("{}/{}", self.base_url, page)
    }

    async fn serve_loop(listener: TcpListener) {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                continue;
            };
            tokio::spawn(async move {
                let mut buf = vec![0u8; 4096];
                let n = match stream.read(&mut buf).await {
                    Ok(n) if n > 0 => n,
                    _ => return,
                };
                let request = String::from_utf8_lossy(&buf[..n]);

                // Parse the request path from "GET /path HTTP/1.1"
                let path = request
                    .lines()
                    .next()
                    .and_then(|line| line.split_whitespace().nth(1))
                    .unwrap_or("/");

                let (status, body) = match path.trim_start_matches('/') {
                    "simple.html" => ("200 OK", SIMPLE_HTML),
                    "form.html" => ("200 OK", FORM_HTML),
                    "table.html" => ("200 OK", TABLE_HTML),
                    "dynamic.html" => ("200 OK", DYNAMIC_HTML),
                    _ => ("404 Not Found", "<h1>404</h1>"),
                };

                let response = format!(
                    "HTTP/1.1 {}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status,
                    body.len(),
                    body,
                );
                let _ = stream.write_all(response.as_bytes()).await;
                let _ = stream.flush().await;
            });
        }
    }
}
