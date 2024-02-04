use messaging_playground::{client, shared};
use prost::Message;
use std::io::{stdout, Write};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};

const TCP_ADDR: &str = "localhost:3000";
const QUEUE_SIZE: usize = 16;

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect(TCP_ADDR).await.unwrap();
    let (mut reader, mut writer) = stream.split();
    let (tx, mut rx) = mpsc::channel(QUEUE_SIZE);

    let set_username_line: String = client::ask_username();
    let mut buf: Vec<u8> = Vec::new();

    shared::models::Message {
        message: set_username_line,
        sender: String::new(),
        msg_type: shared::models::message::Type::Command as i32,
        recipient: None,
    }
    .encode(&mut buf)
    .unwrap();

    writer
        .write_all(&buf)
        .await
        .expect("Failed to write to stream");

    stdout().flush().expect("Failed to flush stdout");

    client::start_reading_user_entries(tx);

    loop {
        let mut read_buffer = [0; client::BUF_SIZE];

        tokio::select! {
            read_length = reader.read(&mut read_buffer) => client::on_message_received(read_length, read_buffer),
            input_opt = rx.recv() => {
                let buf =
                client::on_mpsc_recv(input_opt).await;

                if let Some(buff) = buf {
                    writer
                    .write_all(&buff)
                    .await
                    .expect("Failed to write to stream");
                stdout().flush().expect("Failed to flush stdout");
                }
            },
        }
    }
}
