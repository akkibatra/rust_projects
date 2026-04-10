use std::collections::HashMap;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
enum Command {
    Set(String, String),
    Get(String),
    Ping,
    Unknown
}

type Db = Arc<Mutex<HashMap<String, String>>>;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    println!("Server listening on port 7878");

    let db: Db = Arc::new(Mutex::new(HashMap::new()));
    
    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db_clone = Arc::clone(&db);

        tokio::spawn(async move {
            if let Err(e) = handle_client(socket, db_clone).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

async fn handle_client(mut stream: TcpStream, db: Db) -> Result<(), Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];

    loop {
        let n = stream.read(&mut buffer).await?;
        if n == 0 { return Ok(()); }

        let input = &buffer[..n];
        if input[0] == b'*' {
            let cmd = parse_resp_array(input);

            match cmd {
                Command::Set(key, value ) => {
                    let mut data = db.lock().await;
                    data.insert(key, value);
                    stream.write_all(b"+OK\r\n").await?;
                }
                Command::Get(key) => {
                    let data = db.lock().await;
                    match data.get(&key) {
                        Some(val) => {
                            let response = format!("${}\r\n{}\r\n", val.len(), val);
                            stream.write_all(response.as_bytes()).await?;
                        }
                        None => stream.write_all(b"$-1\r\n").await?,
                    }
                }
                Command::Ping => {
                    stream.write_all(b"+PONG\r\n").await?;
                }
                _ => stream.write_all(b"-ERR unknown command\r\n").await?,
            }

        }
    }
}

fn parse_resp_array(input: &[u8]) -> Command {
    let s = String::from_utf8_lossy(input);
    let mut lines = s.lines();

    let first_line = lines.next().unwrap_or("");
    if !first_line.starts_with('*') {
        return Command::Unknown;
    }

    let mut parts = Vec::new();
    while let Some(line) = lines.next() {
        if line.starts_with("$"){
            if let Some(value) = lines.next() {
                parts.push(value.to_uppercase());
            }
        }
    }

    match parts.get(0).map(|s| s.as_str()) {
        Some("SET") if parts.len() == 3 => Command::Set(parts[1].to_lowercase(), parts[2].clone()),
        Some("GET") if parts.len() == 2 => Command::Get(parts[1].to_lowercase()),
        Some("PING") => Command::Ping,
        _ => Command::Unknown,
    }
}