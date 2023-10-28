use std::net::SocketAddr;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    sync::{
        broadcast::{self, Receiver, Sender},
        Mutex,
    },
};

struct Client {
    name: String,
    // stream: TcpStream,
    // rx: Receiver<(String, SocketAddr)>,
}

#[tokio::main]
async fn main() {
    let clients: Arc<Mutex<HashMap<String, Client>>> = Arc::new(Mutex::new(HashMap::new()));

    let listener = TcpListener::bind("localhost:3000").await.unwrap();

    let (tx, _rx) = broadcast::channel::<(String, SocketAddr)>(10);

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let rx = tx.subscribe();
        let clients_clone = clients.clone();

        handle_new_client(socket, addr, tx, rx, clients_clone);
    }
}

fn handle_new_client(
    mut socket: TcpStream,
    addr: SocketAddr,
    tx: Sender<(String, SocketAddr)>,
    mut rx: Receiver<(String, SocketAddr)>,
    clients: Arc<Mutex<HashMap<String, Client>>>,
) {
    tokio::spawn(async move {
        insert_client(addr.to_string(), clients.clone()).await;

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
                    let (mut msg, other_addr) = result.unwrap();
                    println!("msg: {}; otheraddr {}", msg,other_addr);

                    let clients_guard = clients.lock().await;
                    let current_client = clients_guard.get(&addr.to_string()).unwrap().name.clone();


                    msg = format!("{}: {}", current_client, msg);
                    // if addr!=other_addr{
                        writer.write_all(msg.as_bytes()).await.unwrap();
                    // }
                }

            }
        }
    });
}

async fn insert_client(addr: String, clients: Arc<Mutex<HashMap<String, Client>>>) {
    println!("Client {} connected", addr);

    let mut clients_guard = clients.lock().await;
    clients_guard.insert(
        addr.to_string(),
        Client {
            name: addr,
            // stream: socket,
            // rx,
        },
    );
}
