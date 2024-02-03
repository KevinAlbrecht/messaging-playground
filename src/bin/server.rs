use messaging_playground::{server, shared, sqlite};
use prost::Message;
use std::net::SocketAddr;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
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

const BUF_SIZE: usize = 4096;
const MAX_MESSAGE_SIZE: usize = 2048;
const TRUNATED_MESSAGE_SIZE: usize = 256;

#[tokio::main]
async fn main() {
    let sqlite_provider = match sqlite::SqliteProvider::new() {
        Ok(provider) => provider,
        Err(e) => {
            println!("Error when creating sqlite provider: {}", e);
            return;
        }
    };

    let arc_sqlite_provider = Arc::new(Mutex::new(sqlite_provider));
    let clients: Arc<Mutex<HashMap<String, Client>>> = Arc::new(Mutex::new(HashMap::new()));
    let listener = TcpListener::bind("localhost:3000").await.unwrap();
    let (tx, _rx) = broadcast::channel::<(String, Option<SocketAddr>, Option<SocketAddr>)>(10);

    loop {
        let (socket, addr) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let rx = tx.subscribe();
        let clients_clone = clients.clone();
        let arc_sqlite_provider = arc_sqlite_provider.clone();

        handle_new_client(socket, addr, tx, rx, clients_clone, arc_sqlite_provider);
    }
}

fn handle_new_client(
    mut socket: TcpStream,
    addr: SocketAddr,
    tx: Sender<(String, Option<SocketAddr>, Option<SocketAddr>)>,
    mut rx: Receiver<(String, Option<SocketAddr>, Option<SocketAddr>)>,
    clients: Arc<Mutex<HashMap<String, Client>>>,
    sqlite_provider: Arc<Mutex<sqlite::SqliteProvider>>,
) {
    tokio::spawn(async move {
        let (reader, mut writer) = socket.split();
        let mut reader = BufReader::new(reader);

        let annonce_msg = insert_client(addr.to_string(), clients.clone()).await;
        tx.send((annonce_msg.clone(), None, None)).unwrap();

        loop {
            let mut read_buffer = [0; BUF_SIZE];

            tokio::select! {
                read_length = reader.read(&mut read_buffer) => {
                    match read_length {
                        Ok(0) => {
                            let annonce = remove_client(addr.to_string(), clients.clone()).await;
                            tx.send((annonce.clone(), None, None)).unwrap();
                            return;
                        }
                        Ok(len) => {
                            let received = shared::models::Message::decode(&read_buffer[..len]).unwrap();
                            let new_message_id = sqlite_provider.lock().await
                                .insert_message(
                                    &received.message,
                                    &received.sender,
                                    &received.recipient.unwrap_or_default(),
                                    received.msg_type,
                                )
                                .unwrap();

                            let mut msg= received.message.trim().to_string();

                            if msg.len() > MAX_MESSAGE_SIZE{
                               msg.truncate(TRUNATED_MESSAGE_SIZE);
                               msg.push_str(&format!("...\n\n Message too long, full message visible with id: {}.",new_message_id));
                            }

                            if !msg.starts_with("/"){
                                tx.send((msg,Some(addr),None)).unwrap();
                            }else{
                                println!("commande: {}-{}",msg,addr);
                                let response = handle_command(msg, addr.to_string(), clients.clone(),sqlite_provider.clone()).await;

                                if let Some(response_msg) = response{
                                    tx.send((response_msg,Some(addr),Some(addr))).unwrap();
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Error when reading from stream: {}", e);
                        }
                    }
                },

                result = rx.recv()=>{
                    let (msg, sender_addr,to_addr) = result.unwrap();

                    let mut current_client:String= "Server".to_string();

                    if let Some(sender_address) = sender_addr {
                        let clients_guard = clients.lock().await;
                        current_client = clients_guard.get(&sender_address.to_string()).unwrap().name.clone();
                    }

                    if to_addr.is_some() {
                        println!("to_addr: {}",to_addr.unwrap());
                        let to_addr = to_addr.unwrap();

                        if to_addr != addr{
                            continue;
                        }
                    }

                    let message = shared::models::Message {
                        message: msg,
                        sender: current_client,
                        msg_type: shared::models::message::Type::Broadcast as i32,
                        recipient: None,
                    };

                    let mut buf = Vec::new();
                    message.encode(&mut buf).unwrap();
                    writer.write_all(&buf).await.unwrap();
                }
            }
        }
    });
}

async fn handle_command(
    input: String,
    sender_addr: String,
    clients: Arc<Mutex<HashMap<String, Client>>>,
    sql_provider: Arc<Mutex<sqlite::SqliteProvider>>,
) -> Option<String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let command_str = parts[0];

    if let Some(command) = server::Command::from_str(command_str) {
        return match command {
            server::Command::Nick => {
                let new_name = parts.get(1).unwrap().to_string();
                update_nickname(new_name, clients, sender_addr).await;
                None
            }
            server::Command::List => {
                let list = list_clients(clients).await;
                Some(list)
            }
            server::Command::Read => {
                let id = parts[1].to_string().parse::<u16>().unwrap();
                let message = get_message_by_id(id, sql_provider).await;
                message
            }
        };
    }
    None
}

// TODO return a Message instead of a String
async fn get_message_by_id(
    id: u16,
    sqlite_provider: Arc<Mutex<sqlite::SqliteProvider>>,
) -> Option<String> {
    sqlite_provider.lock().await.get_message_by_id(id)
}

async fn insert_client(addr: String, clients: Arc<Mutex<HashMap<String, Client>>>) -> String {
    let mut clients_guard = clients.lock().await;
    clients_guard.insert(
        addr.to_string(),
        Client {
            name: addr.to_string(),
        },
    );

    println!("Client {}  joined", addr);

    format!("{} joined", addr)
}

async fn remove_client(addr: String, clients: Arc<Mutex<HashMap<String, Client>>>) -> String {
    let current_client_name: String;

    let mut clients_guard = clients.lock().await;
    {
        current_client_name = clients_guard.get(&addr.to_string()).unwrap().name.clone();
    }

    println!(
        "Client {} alias \"{}\" disconnected",
        addr, current_client_name
    );

    clients_guard.remove(&addr.to_string());

    format!("{} left", current_client_name)
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
    for (_, current_client) in clients_guard.iter() {
        list.push_str(&current_client.name);
        list.push_str("\n");
    }

    list
}
