use std::{
    io::{stdin, stdout, Write},
    process::exit,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{tcp::ReadHalf, tcp::WriteHalf, TcpStream},
    sync::mpsc,
};

pub mod chat {
    pub mod message {
        include!(concat!(env!("OUT_DIR"), "/chat.message.rs"));
    }
}

use chat::message::{self, message::Type};
use prost::Message;

const TCP_ADDR: &str = "localhost:3000";
const BUF_SIZE: usize = 4096;
const QUEUE_SIZE: usize = 16;

#[tokio::main]
async fn main() {
    let set_username_line: String = ask_username();

    let mut stream = TcpStream::connect(TCP_ADDR).await.unwrap();
    let (reader, mut writer) = stream.split();
    let (tx, rx) = mpsc::channel(QUEUE_SIZE);

    writer
        .write_all(set_username_line.as_bytes())
        .await
        .expect("Failed to write to stream");
    stdout().flush().expect("Failed to flush stdout");

    start_reading_user_entries(tx);
    start_tcp_read_write(reader, &mut writer, rx).await;
}

fn start_reading_user_entries(tx: mpsc::Sender<String>) {
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
                        // let received = String::from_utf8_lossy(&read_buffer[..len]);
                        let received = message::Message::decode(&read_buffer[..len]).unwrap();

                        // println!("msg:{}\nsender:{}\n,mgs_type:{}\nrecipient:{}",
                        // received.message.to_string(),
                        // received.sender.to_string(),
                        // received.msg_type.to_string(),
                        // if received.recipient.is_some() {received.recipient.unwrap()} else {"no recipiend".to_string()});

                        println!("{}: {}", received.sender, received.message);
                    }
                    Err(e) => {
                        eprintln!("Error when reading from stream: {}", e);
                        exit(0)
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
        print!("\x1b[1A\x1b[K");

        buffer
    })
    .await
    .expect("Failed to join task")
}

fn ask_username() -> String {
    let mut username = String::new();
    println!("Enter your username: ");
    stdin().read_line(&mut username).unwrap();

    let mut name = "/nick ".to_string();
    name.push_str(username.trim());
    name.push_str("\r\n");

    name
}
