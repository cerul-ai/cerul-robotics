use serde_json::Value;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    thread,
    time::Duration,
};
pub(crate) fn server(
    responses: Vec<(u16, Value)>,
) -> (String, thread::JoinHandle<Vec<(String, Value)>>) {
    server_with_retry_header(responses, true)
}
fn server_with_retry_header(
    responses: Vec<(u16, Value)>,
    retry_header: bool,
) -> (String, thread::JoinHandle<Vec<(String, Value)>>) {
    let count = responses.len();
    let mut responses = responses.into_iter();
    scripted_server(count, retry_header, move |_| responses.next().unwrap())
}
pub(crate) fn scripted_server(
    count: usize,
    retry_header: bool,
    response: impl FnMut(&Value) -> (u16, Value) + Send + 'static,
) -> (String, thread::JoinHandle<Vec<(String, Value)>>) {
    scripted_server_with_body_delay(count, retry_header, Duration::ZERO, response)
}
fn scripted_server_with_body_delay(
    count: usize,
    retry_header: bool,
    body_delay: Duration,
    mut response: impl FnMut(&Value) -> (u16, Value) + Send + 'static,
) -> (String, thread::JoinHandle<Vec<(String, Value)>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    listener.set_nonblocking(true).unwrap();
    let handle = thread::spawn(move || {
        let mut requests = Vec::new();
        for _ in 0..count {
            let start = std::time::Instant::now();
            let mut socket = loop {
                match listener.accept() {
                    Ok((socket, _)) => break socket,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(
                            start.elapsed() < Duration::from_secs(15),
                            "request was not received"
                        );
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(e) => panic!("{e}"),
                }
            };
            socket.set_nonblocking(false).unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut reader = BufReader::new(socket.try_clone().unwrap());
            let mut first = String::new();
            reader.read_line(&mut first).unwrap();
            let mut length = 0;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" {
                    break;
                }
                if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
                    length = value.trim().parse::<usize>().unwrap();
                }
            }
            let mut bytes = vec![0; length];
            reader.read_exact(&mut bytes).unwrap();
            requests.push((
                first,
                serde_json::from_slice(&bytes)
                    .unwrap_or(Value::String(String::from_utf8_lossy(&bytes).into_owned())),
            ));
            let (status, body) = response(&requests.last().unwrap().1);
            let body = body.to_string();
            let retry = if retry_header {
                "Retry-After: 0\r\n"
            } else {
                ""
            };
            write!(socket,"HTTP/1.1 {status} Test\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{retry}Connection: close\r\n\r\n",body.len()).unwrap();
            socket.flush().unwrap();
            thread::sleep(body_delay);
            write!(socket, "{body}").unwrap();
        }
        requests
    });
    (format!("http://{address}/v1beta"), handle)
}
