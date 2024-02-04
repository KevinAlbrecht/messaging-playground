use arboard::Clipboard;
use std::{
    future::Future,
    io::{stdin, stdout, Error, Write},
    process::exit,
};
use tokio::sync::mpsc;
use tokio::task::futures;

use crate::shared;
use prost::Message;

pub const BUF_SIZE: usize = 4096;

enum Command {
    Paste,
}

impl Command {
    pub fn from_str(s: &str) -> Option<Command> {
        match s {
            "/paste" => Some(Command::Paste),
            _ => None,
        }
    }
}

pub fn ask_username() -> String {
    let mut username = String::new();
    println!("Enter your username: ");
    stdin().read_line(&mut username).unwrap();

    let mut name = "/nick ".to_string();
    name.push_str(username.trim());
    name.push_str("\r\n");

    name
}

pub fn handle_command(input: String) -> String {
    if input.starts_with('/') {
        let parts: Vec<&str> = input.split_whitespace().collect();
        let command_str = parts[0];

        if let Some(command) = Command::from_str(command_str) {
            return match command {
                Command::Paste => {
                    let mut clipboard = Clipboard::new().unwrap();
                    clipboard.get_text().unwrap()
                }
            };
        }
    }

    input
}

pub fn start_reading_user_entries(tx: mpsc::Sender<String>) {
    tokio::spawn(async move {
        loop {
            let input = handle_command(read_user_entry().await).trim().to_string();
            tx.send(input).await.expect("Failed to send user input");
        }
    });
}

pub async fn read_user_entry() -> String {
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

pub fn on_message_received(len: Result<usize, Error>, buf: [u8; BUF_SIZE]) {
    return match len {
        Ok(0) => {
            println!("Connection closed by server.");
            return;
        }
        Ok(len) => {
            let received = shared::models::Message::decode(&buf[..len]).unwrap();
            println!("{}: {}", received.sender, received.message);
        }
        Err(e) => {
            eprintln!("Error when reading from stream: {}", e);
            exit(0)
        }
    };
}

pub async fn on_mpsc_recv(input_opt: Option<String>) -> Option<Vec<u8>> {
    if let Some(input) = input_opt {
        let mut buf = Vec::new();

        shared::models::Message {
            message: input,
            sender: String::new(),
            msg_type: shared::models::message::Type::Broadcast as i32,
            recipient: None,
        }
        .encode(&mut buf)
        .unwrap();

        return Some(buf);
    }

    None
}
