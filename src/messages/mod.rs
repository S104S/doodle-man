use std::alloc::handle_alloc_error;
use std::convert::TryInto;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

pub struct Message {

}

impl Message {
    pub fn listen() -> String {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

        for stream in listener.incoming() {
            println!("Received something...");
            Self::handle_client(stream.unwrap());
        }
        "Message received".to_string()
    }

    fn handle_client(mut stream: TcpStream) {
        let mut stream_buf = String::new();
        stream.read_to_string(&mut stream_buf).expect("Error reading stream");
        println!("{}", stream_buf);
    }
}