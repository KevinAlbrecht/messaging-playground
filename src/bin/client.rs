use std::fmt::write;
use std::io::stdin;
use std::io::stdout;
use std::io::Write;
use std::net::SocketAddr;
use std::str;

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::broadcast::{self, Receiver, Sender},
};

const TCP_ADDR: &str = "127.0.0.1:3000";

// #[tokio::main]
async fn main2() {
    let mut stream = TcpStream::connect("localhost:3000").await.unwrap();

    let mut read_buffer = Vec::new();

    loop {
        let (mut reader, mut writer) = stream.split();
        let entry: String;

        tokio::select! {
            read_length = reader.read_to_end(&mut read_buffer)=>{

                println!("LOOOOO {:?}",read_length);

                match read_length {
                    Ok(_)=>{
                        let lowercase_str = String::from_utf8(read_buffer.clone()).unwrap_or_default().to_ascii_lowercase();
                        println!(":{}", lowercase_str);
                    }
                    Err(e) =>{
                        eprintln!("Error when reading stream: {}",e);
                    }
                }
            }
            bytes_written = {
                entry = read_user_entry().await;
                writer.write_all(entry.as_bytes())
            }=>{
                match bytes_written {
                    Ok(_)=>{
                        stream.flush().await;
                    }
                    Err(e)=>{
                        eprintln!("Error when writting on stream: {}",e);
                    }
                }

                }
        }

        // let mut read_buffer = Vec::new();
        // stream.read(&mut read_buffer).await;

        // println!(":{:?}", String::from_utf8(read_buffer.to_ascii_lowercase()));

        // loop {
        //     let entry = read_user_entry();
        //     let writter_bytes = stream.write(entry.as_bytes()).await.unwrap();

        //     if writter_bytes < entry.len() {
        //         println!("Data lost");
        //     } else {
        //         stream.flush();
        //     }
        // }
    }
}

async fn read_user_entry() -> String {
    return tokio::task::spawn_blocking(|| {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();

        print!("\x1b[1A\x1b[K");
        stdout().flush().unwrap();

        return buffer;
    })
    .await
    .unwrap();
}

#[tokio::main]
async fn main() {
    let mut stream = TcpStream::connect(TCP_ADDR).await.unwrap();
    let (reader, mut writer) = stream.split();

    let read_handle = tokio::spawn(async move {
        // let (reader, mut writer) = stream.split();
        let mut reader = BufReader::new(reader);
        loop {
            let mut buf = String::new();
            if reader.read_line(&mut buf).await.unwrap() > 0 {
                println!("→{}", buf.trim());
            }
        }
    });

    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf);
        writer.write_all(buf.as_bytes()).await;
    }
}
