use std::io::{stdin, stdout, Write};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{tcp::ReadHalf, tcp::WriteHalf, TcpStream},
    sync::mpsc,
};

const TCP_ADDR: &str = "localhost:3000";
const BUF_SIZE: usize = 4096;
const QUEUE_SIZE: usize = 16;

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect(TCP_ADDR).await.unwrap();
    let (reader, mut writer) = stream.split();
    let (tx, rx) = mpsc::channel(QUEUE_SIZE);

    start_reading_user_entries(tx);
    start_tcp_read_write(reader, &mut writer, rx).await;
}

fn start_reading_user_entries(tx: mpsc::Sender<String>) {
    // Tâche pour lire les entrées de l'utilisateur
    tokio::spawn(async move {
        loop {
            let input = read_user_entry().await;
            tx.send(input).await.expect("Failed to send user input");
        }
    });
}

async fn start_tcp_read_write(
    mut reader: ReadHalf<'_>,
    writer: &mut WriteHalf<'_>,
    mut rx: mpsc::Receiver<String>,
) {
    loop {
        let mut read_buffer = [0; BUF_SIZE];

        tokio::select! {
            read_length = reader.read(&mut read_buffer) => {
                match read_length {
                    Ok(0) => {
                        println!("Connection closed by server.");
                        return;
                    }
                    Ok(len) => {
                        let received = String::from_utf8_lossy(&read_buffer[..len]);
                        println!("{}: {}", "Toto", received);
                    }
                    Err(e) => {
                        eprintln!("Error when reading from stream: {}", e);
                    }
                }
            }
            input_opt = rx.recv() => {
                if let Some(input) = input_opt {
                    writer.write_all(input.as_bytes()).await.expect("Failed to write to stream");
                    stdout().flush().expect("Failed to flush stdout");
                }
            }
        }
    }
}

async fn read_user_entry() -> String {
    stdout().flush().expect("Failed to flush stdout");
    tokio::task::spawn_blocking(|| {
        let mut buffer = String::new();
        stdin()
            .read_line(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    })
    .await
    .expect("Failed to join task")
}
