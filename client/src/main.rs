use std::net::{TcpStream};
use std::process::exit;
use std::io;
use std::thread;
use std::io::{Read, Write};

fn main() {
    let mut logged_in = false;
    let mut connection = connect_to_server();
    let listener = connection.try_clone().unwrap();
    let _listener_handle = thread::spawn(move || {
        message_receiver(listener);
    });
    loop {
        parse_command(&mut connection, &mut logged_in);
    } 
}

fn message_receiver(mut connection: TcpStream) {
    loop {
        let mut buffer : [u8;8096] = [0;8096];
        let msg_result = connection.read(&mut buffer);
        let bytes = match msg_result { 
            Ok(size) =>  size, //println!("Received {size} bytes from server"),
            Err(err) => { 
                eprintln!("Recv: {err}");
                return;
            }
        };
        let msg = String::from_utf8(buffer.to_vec()).unwrap_or_else(|err| {
            eprintln!("Error converting to str: {err}");
            String::from("error")
        });
        if bytes > 0 {
            println!("{msg}");
        }
    }
}
fn parse_command(connection: &mut TcpStream, logged_in: &mut bool) -> i32 {
    let mut command = String::new();
    io::stdin().read_line(&mut command)
        .expect("If read_line() fails god help you");
    let mut words = command.split_whitespace();
    let first_word=  words.next().unwrap_or_else( || {
        "error"
    });
    if first_word.eq("exit") {
        exit(0)
    } else if first_word.eq("msg") {
        hangle_msg(connection, &command, logged_in);
    } else if first_word.eq("login") {
        handle_login(connection, &command, logged_in);
    } else {
        eprintln!("Unknown command!")
    }
    0
}
fn handle_login(connection: &mut TcpStream, command: &String, logged_in: &mut bool) -> i32 {
    //TODO: "error" cannot be used as a login
    let mut words = command.split_whitespace();
    let _first_word = words.next().unwrap_or_else( || {
        "error"
    });
    if *logged_in {
        eprintln!("Already logged in!");
        return 1
    }
    let login = words.next().unwrap_or_else(|| {
        eprintln!("Usage: login <login>");
        "error"
    });
    if login.eq("error") {return -1}
    println!("Current login: {login}");
    connection.write(login.as_bytes()).unwrap_or_else(|err| {
        eprintln!("login: {err}");
        1
    });
    *logged_in = true;
    0
}

fn hangle_msg(connection: &mut TcpStream, command: &String, logged_in: &bool) -> i32 {
    let mut words = command.split_whitespace();
    let _first_word = words.next().unwrap_or_else( || {
        "error"
    });
    if !logged_in {
        eprintln!("Cannot message without logging in!");
        return 1
    }
    let receiver =  words.next().unwrap_or_else(|| {
        eprintln!("Usage: msg <login> <content of the message>");
        "error"
    });
    if receiver.eq("error") {return 1}
    let test_word = words.next().unwrap_or_else(|| {
        eprintln!("Message must contain at least a single word!");
        "error"
    });
    if test_word.eq("error") {return 1}
    let msg = &command[4..];
    println!("{msg}");
    connection.write(msg.as_bytes()).unwrap_or_else(|err| {
        eprintln!("login: {err}");
        0
    });
    0
}
fn connect_to_server() -> TcpStream {
    let connection = TcpStream::connect("127.0.0.1:2137").unwrap_or_else(|err| {
        eprintln!("Connect: {err}");
        exit(1);
    });
    connection
}

