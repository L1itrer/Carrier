use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use std::io::{stdout, Read, Write};
use std::ptr::write;

type Users = Arc<Mutex<HashMap<String, String>>>;


fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:2137")?;

    let users : Users = Arc::new(Mutex::new(HashMap::new()));
    let mut connections = Vec::new();
    for stream in listener.incoming() {
        match stream {
            Ok(socket) => {
                let users = users.clone();
                let handle = thread::spawn(move || {
                   handle_connection(socket, users);
                });
                connections.push(handle);
            }
            Err(e) => {
                eprintln!("Accept: {e}")
            }
        }
    }


    Ok(())
}

fn handle_connection(mut stream: TcpStream, users: Users) {
    loop {
        let mut buffer= [0; 1024];
        
        let result = stream.read(&mut buffer).unwrap_or_else(|err| {
            eprintln!("Recv: {err}");
            0
        });
        if result == 0 {
            return;
        }
        stdout().write(&buffer).unwrap_or_else(|err| {
            eprintln!("write: {err}");
            panic!()
        });
    }
}

