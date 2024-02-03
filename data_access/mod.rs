pub mod sqlite_provider {
    use std::fs;

    use rusqlite::{params, Connection, Result};

    pub mod chat {
        pub mod message {
            include!(concat!(env!("OUT_DIR"), "/chat.message.rs"));
        }
    }

    pub struct SqliteProvider {
        conn: Connection,
    }

    const DB_PATH: &str = "databases";
    const DB_FILE_NAME: &str = "chat.db";

    impl SqliteProvider {
        pub fn new() -> Result<Self> {
            if !fs::metadata(DB_PATH).is_ok() {
                match fs::create_dir(DB_PATH) {
                    Ok(_) => println!("Database directory created"),
                    Err(e) => {
                        println!("Error when creating database directory: {}", e);
                        panic!("Error when creating database directory")
                    }
                }
            }

            let db_file_full_path = format!("{}/{}", DB_PATH, DB_FILE_NAME);

            let conn = match Connection::open(db_file_full_path) {
                Ok(conn) => conn,
                Err(e) => {
                    println!("Error when opening database: {}", e);
                    panic!("Error when opening database")
                }
            };

            conn.execute(
                "CREATE TABLE IF NOT EXISTS messages (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    message TEXT NOT NULL,
                    sender TEXT NOT NULL,
                    recipient TEXT NULL,
                    msg_type INT NOT NULL DEFAULT 0
                )",
                params![],
            )?;
            Ok(Self { conn })
        }

        pub fn insert_message(
            &self,
            message: &str,
            sender: &str,
            recipient: &str,
            msg_type: i32,
        ) -> Result<i64> {
            {
                match self.conn.execute(
                    "INSERT INTO messages (message,sender,recipient,msg_type) VALUES (?,?,?,?)",
                    params![message, sender, recipient, msg_type as i32],
                ) {
                    Ok(_) => Ok(self.conn.last_insert_rowid()),
                    Err(e) => {
                        println!("Error when inserting message: {}", e);
                        Err(e)
                    }
                }
            }
        }

        pub fn get_message(&self, id: u16) -> Result<String> {
            self.conn
                .query_row("SELECT * FROM messages WHERE id = ?", params![id], |row| {
                    row.get(0)
                })
        }

        pub fn count_messages(&self) -> Result<i64> {
            self.conn
                .query_row("SELECT COUNT(*) FROM messages", params![], |row| row.get(0))
        }
    }
}
