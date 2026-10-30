use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpListener, task::JoinHandle};
use url::Url;

pub struct Reply {
    pub status: &'static str,
    pub headers: &'static str,
    pub body: Vec<u8>,
}

impl Reply {
    pub fn ok(body: impl Into<Vec<u8>>) -> Self {
        Self { status: "200 OK", headers: "", body: body.into() }
    }
}

pub async fn fixture_server(replies: Vec<Reply>) -> (Url, JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind fixture listener");
    let address = listener.local_addr().expect("fixture address");
    let base_url = Url::parse(&format!("http://{address}/")).expect("fixture URL");
    let task = tokio::spawn(async move {
        let mut requests = Vec::new();
        for reply in replies {
            let (mut stream, _) = listener.accept().await.expect("accept request");
            let mut request = Vec::new();
            let mut chunk = [0_u8; 1024];
            loop {
                let count = stream.read(&mut chunk).await.expect("read request");
                if count == 0 { break; }
                request.extend_from_slice(&chunk[..count]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") { break; }
            }
            requests.push(String::from_utf8_lossy(&request).into_owned());
            let head = format!(
                "HTTP/1.1 {}\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n",
                reply.status,
                reply.body.len(),
                reply.headers,
            );
            stream.write_all(head.as_bytes()).await.expect("write response headers");
            stream.write_all(&reply.body).await.expect("write response body");
            stream.flush().await.expect("flush response");
        }
        requests
    });
    (base_url, task)
}
