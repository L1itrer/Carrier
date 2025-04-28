use std::net::{TcpStream};
use std::process::exit;
use std::io;

fn main() {
    let connection = TcpStream::connect("127.0.0.1:2137").unwrap_or_else(|err| {
        eprintln!("Connect: {err}");
        exit(1);
    });
    loop {
        let mut command = String::new();
        io::stdin().read_line(&mut command)
            .expect("If read_line() fails god help you");
        let mut words = command.split_whitespace();
        let first_word=  words.next().unwrap_or_else( || {
            "error"
        });
        if first_word.eq("exit") {
            break
        } else if first_word.eq("msg") {
            let reciever =  words.next().unwrap_or_else(|| {
                eprintln!("Usage: msg <login> <content of the message>");
                "error"
            });
            if reciever.eq("error") {continue;}
            let test_word = words.next().unwrap_or_else(|| {
                eprintln!("Message must contain at least a single word!");
                "error"
            });
            if test_word.eq("error") {continue;}
            let msg = &command[(4+reciever.len()+1)..];
            println!("{msg}");
        } else if first_word.eq("login") {

        }
        else {
            eprintln!("Unknown command!")
        }

    }
}