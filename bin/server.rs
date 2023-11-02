use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};

use bytes::Bytes;
use mini_redis::{Command, Connection, Frame};
use mini_redis::Command::{Get, Set};
use tokio::net::{TcpListener, TcpStream};

type Db = Arc<Mutex<HashMap<String, Bytes>>>;
type ShardedDb = Arc<Vec<Mutex<HashMap<String, Bytes>>>>;

fn new_sharded_db(num_shards: usize) -> ShardedDb {
    let mut db = Vec::with_capacity(num_shards);
    for _ in 0..num_shards {
        db.push(Mutex::new(HashMap::new()));
    }
    Arc::new(db)
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    let db = new_sharded_db(10);

    loop {
        let (socket, _) = listener.accept().await.unwrap();
        let db = db.clone();
        tokio::spawn(async move {
            process(socket, db).await;
        });
    }
}

async fn process(socket: TcpStream, db: ShardedDb) {
    let mut connection = Connection::new(socket);
    while let Some(frame) = connection.read_frame().await.unwrap() {
        let response = match Command::from_frame(frame).unwrap() {
            Set(cmd) => {
                let key = cmd.key();
                let mut sharded = db[key.len() % db.len()].lock().unwrap();
                sharded.insert(key.to_string(), cmd.value().clone());
                Frame::Simple("OK".to_string())
            }
            Get(cmd) => {
                let key = cmd.key();
                let mut sharded = db[key.len() % db.len()].lock().unwrap();
                if let Some(value) = sharded.get(key) {
                    Frame::Bulk(value.clone())
                } else {
                    Frame::Null
                }
            }
            cmd => panic!("unimplemented {:?}", cmd)
        };
        connection.write_frame(&response).await.unwrap();
    }
}