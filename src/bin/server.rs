use std::net::SocketAddr;

use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::broadcast::{self, Receiver, Sender},
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:3000").await.unwrap();

    let (tx, _rx) = broadcast::channel::<(String, SocketAddr)>(10);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    read = reader.read_line(&mut line) =>{
                        if read.unwrap() == 0 {
                            break;
                        }

                        println!("→: {}", line);
                        tx.send((line.clone(),addr)).unwrap();
                        line.clear();
                    }
                    result = rx.recv()=>{
                        let (msg, other_addr) = result.unwrap();
                        // if(addr!=other_addr){
                        writer.write_all(msg.as_bytes()).await.unwrap();
                        // }
                    }

                }
            }
        });
        // handle_connection(&mut socket, &tx, &mut rx).await;
    }
}

// async fn handle_connection(socket: &mut TcpStream, tx: &Sender<String>, rx: &mut Receiver<String>) {

// }
