use std::net::{TcpListener, TcpStream};
use std::thread;
use std::sync::{Mutex, Arc};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::collections::VecDeque;
type UsersCollection = Arc<Mutex<HashMap<String, User>>>;
struct User {
    msg_queue: VecDeque<String>,
    socket: TcpStream,
    is_online: bool,
}

fn main() -> std::io::Result<()>{
    let listener = TcpListener::bind("127.0.0.1:2137")?;

    let mut users : UsersCollection = Arc::new(Mutex::new(HashMap::new()));
    let mut connections = Vec::new();
    println!("Server running");
    for stream in listener.incoming() {
        println!("A new client connected!");
        match stream {
            Ok(mut socket) => {
                let mut buffer: [u8;1024] = [0; 1024];
                let login = msg_get_login(&mut buffer, &mut socket);
                if login.eq("error") {
                    eprintln!("Did not receive the login from the user");
                    continue
                }
                println!("Received login: \"{login}\"");
                user_add_if_not_exists(&mut users, login.clone(), socket.try_clone()?);
                let users = users.clone();
                let handle = thread::spawn(move || {
                   handle_connection(socket, users, login);
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

fn handle_connection(mut stream: TcpStream, mut users: UsersCollection, mut login: String) {
    let mut buffer: [u8;1024] = [0; 1024];
    if user_has_pending_messages(&mut users, &login) {
        user_send_pending_messages(&mut users, &login, &mut stream);
    }
    loop {
        let received_bytes = stream.read(&mut buffer).unwrap_or_else(|err| {
             eprintln!("Recv: {err}");
            0
        });
        if received_bytes <= 0 {break}
        let msg = String::from_utf8(buffer.to_vec()).unwrap_or_else(|err| {
            eprintln!("Error converting to str: {err}");
            String::from("error")
        });
        println!("Entire msg: {msg}");
        if msg.eq("error") {break}
        if msg.eq("end") {break}
        let mut words = msg.split_whitespace();
        let target = words.next().unwrap().trim();
        println!("Detected target: \"{target}\"");
        let mut target_login = String::new();
        target_login += target;
        if !user_exists(&target_login, &users) {
            println!("Target \"{target_login}\" does not exist");
            writeln!(stream, "User {target_login} does not exist").unwrap_or_else(|err| {
                eprintln!("Error while sending that the client does not exist: {err}");
            });
            continue
        }
        let msg_without_target = &msg[target.len()..];
        println!("Msg without target: {msg_without_target}");
        send_msg(&mut login, target_login, &mut users, String::from(msg_without_target));
    }
    println!("Ending communication with client");
    user_update_online(&mut users, &login, false);
}

fn msg_get_login(buffer: &mut [u8], stream: &mut TcpStream) -> String {
    let received_bytes = stream.read(buffer).unwrap_or_else(|err| {
        eprintln!("Recv: {err}");
        0
    });
    if received_bytes <= 0 {return String::from("error")}
    let msg = String::from_utf8(buffer[..received_bytes].to_vec()).unwrap_or_else(|err| {
        eprintln!("Error converting to str: {err}");
        String::from("error")
    });
    msg
}

fn user_exists(login: &String, users: &UsersCollection) -> bool {
    let data = users.lock().unwrap();
    data.contains_key(login)
}

fn user_add(users: &mut UsersCollection, login: String, socket: TcpStream) -> i32 {
    let mut data = users.lock().unwrap();
    let user = User {msg_queue: VecDeque::new(), is_online: true, socket };
    data.insert(login, user);
    0
}
fn user_add_if_not_exists(users: &mut UsersCollection, login: String, socket: TcpStream) -> i32{
    if !user_exists(&login, users) {
        println!("Added user with login \"{login}\"");
        user_add(users, login, socket);
        return 0
    }
    user_update_online(users, &login, true);
    user_update_connection(users, &login, socket);
    0
}

fn user_update_online(users: &mut UsersCollection, login: &String, value: bool) -> i32 {
    let mut data = users.lock().unwrap();
    let user = data.get_mut(login).unwrap();
    user.is_online = value;
    0
}

fn user_update_connection(users: &mut UsersCollection, login: &String, socket: TcpStream) -> i32 {
    let mut data = users.lock().unwrap();
    let user = data.get_mut(login).unwrap();
    user.socket = socket;
    0
}

fn user_has_pending_messages(users: &mut UsersCollection, login: &String) -> bool {
    let mut data = users.lock().unwrap();
    let user_opt = data.get_mut(login);
    let user;
    match user_opt {
        Some(usr) => user = usr,
        None => return false
    }
    !user.msg_queue.is_empty()
}

fn user_send_pending_messages(users: &mut UsersCollection, login: &String, stream: &mut TcpStream) {
    let mut data = users.lock().unwrap();
    let user = data.get_mut(login).unwrap();
    let queue = &mut user.msg_queue;
    while !queue.is_empty() {
        let message = queue.pop_front().unwrap();
        write!(stream, "{message}").unwrap_or_else(|err| {
            eprintln!("Error in sending pending messages: {err}");
        });
    }
}
fn send_msg(from: &mut String, to: String, users: &mut UsersCollection, msg: String) {
    let msg = format!("[{}]: {}", from.clone(), msg);
    let mut data = users.lock().unwrap();
    let receiver = data.get_mut(&to).unwrap();
    println!("{msg}");
    if receiver.is_online {
        let connection = &mut receiver.socket;
        write!(connection, "{msg}").unwrap_or_else(|err| {
            eprintln!("send_msg(): {err}");
        });
    } else {
        receiver.msg_queue.push_back(msg);
    }
}