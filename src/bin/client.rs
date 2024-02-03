use arboard::Clipboard;
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
    let mut stream = TcpStream::connect(TCP_ADDR).await.unwrap();
    let (reader, mut writer) = stream.split();
    let (tx, rx) = mpsc::channel(QUEUE_SIZE);

    let set_username_line: String = ask_username();
    let mut buf: Vec<u8> = Vec::new();
    
    message::Message {
        message: set_username_line,
        sender: String::new(),
        msg_type: Type::Command as i32,
        recipient: None,
    }
    .encode(&mut buf)
    .unwrap();

    writer
        .write_all(&buf)
        .await
        .expect("Failed to write to stream");

    stdout().flush().expect("Failed to flush stdout");

    start_reading_user_entries(tx);
    start_tcp_read_write(reader, &mut writer, rx).await;
}

fn start_reading_user_entries(tx: mpsc::Sender<String>) {
    tokio::spawn(async move {
        loop {
            let input = handle_command(read_user_entry().await).trim().to_string();
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
                        let received = message::Message::decode(&read_buffer[..len]).unwrap();
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
                    let mut buf = Vec::new();

                    message::Message {
                        message: input,
                        sender: String::new(),
                        msg_type: Type::Broadcast as i32,
                        recipient: None,
                    }.encode(&mut buf).unwrap();

                    writer.write_all(&buf).await.expect("Failed to write to stream");
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

fn handle_command(input: String) -> String {

    if !input.starts_with('/') {
        return input;
    }

    let potential_command = input.replace("\n", " ");
    let parts = potential_command.split(" ").collect::<Vec<&str>>();

    match parts[0] {
        "/paste" => {
            let mut clipboard = Clipboard::new().unwrap();
            return clipboard.get_text().unwrap();
        }
        _ => input,
    }
}
