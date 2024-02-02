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

    let (tx, _rx) = broadcast::channel::<(String, SocketAddr, Option<SocketAddr>)>(10);

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
    tx: Sender<(String, SocketAddr, Option<SocketAddr>)>,
    mut rx: Receiver<(String, SocketAddr, Option<SocketAddr>)>,
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
                        println!("Client {} disconnected", addr);
                        break;
                    }

                    println!("{}: {}", addr,line);
                    let trimmed = line.trim().to_string();

                    if !trimmed.starts_with("/"){
                        tx.send((trimmed,addr,None)).unwrap();
                    }else{
                        println!("commande: {}-{}",trimmed,addr);
                        let response = handle_command(trimmed, addr.to_string(), clients.clone()).await;
                        if let Some(msg) = response{
                            tx.send((msg,addr,Some(addr))).unwrap();
                        }
                    }

                    line.clear();
                }
                result = rx.recv()=>{
                    let (mut msg, sender_addr,to_addr) = result.unwrap();

                    let clients_guard = clients.lock().await;
                    let current_client = clients_guard.get(&sender_addr.to_string()).unwrap().name.clone();

                    msg = format!("{}: {}", current_client, msg);

                    if to_addr.is_some() {
                        println!("to_addr: {}",to_addr.unwrap());
                        let to_addr = to_addr.unwrap();
                        if to_addr == addr{
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }else{
                        writer.write_all(msg.as_bytes()).await.unwrap();
                    }
                }
            }
        }
    });
}

async fn handle_command(
    command: String,
    sender_addr: String,
    clients: Arc<Mutex<HashMap<String, Client>>>,
) -> Option<String> {
    match command.split(" ").collect::<Vec<&str>>()[0] {
        "/nick" => {
            let new_name = command[6..].to_string().to_string();

            update_nickname(new_name, clients, sender_addr).await;
            None
        }
        "/list" => {
            let list = list_clients(clients).await;
            Some(list)
        }
        _ => None,
    }
}

async fn insert_client(addr: String, clients: Arc<Mutex<HashMap<String, Client>>>) {
    println!("Client {} connected", addr);

    let mut clients_guard = clients.lock().await;

    clients_guard.insert(
        addr.to_string(),
        Client {
            name: addr.to_string(),
            // stream: socket,
            // rx,
        },
    );
}

async fn update_nickname(
    new_name: String,
    clients: Arc<Mutex<HashMap<String, Client>>>,
    sender_addr: String,
) {
    let mut clients_guard = clients.lock().await;
    let current_client = clients_guard.get_mut(&sender_addr.to_string()).unwrap();
    current_client.name = new_name;
}

async fn list_clients(clients: Arc<Mutex<HashMap<String, Client>>>) -> String {
    let clients_guard = clients.lock().await;

    let mut list = String::new();
    for (addr, current_client) in clients_guard.iter() {
        list.push_str(&current_client.name);
        list.push_str("\n");
    }

    list
}
