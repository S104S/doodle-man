use std::alloc::handle_alloc_error;
use std::convert::TryInto;
use std::io::{Read, Write};
use std::net::{SocketAddrV6, TcpListener, TcpStream, ToSocketAddrs};
use std::thread;
use reqwest::{Client, Url};

pub struct Message {}

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


    pub fn handle_incoming_messages(mut stream: &TcpStream) -> bool {
        // let mut stream_buf = String::new();
        // stream.read_to_string(&mut stream_buf).expect("Error reading stream");
        // println!("{}", stream_buf);

        loop {
            // println!("Inside loop...");
            let mut twitch_message: String = String::new();
            let result = stream.read_to_string(&mut twitch_message);
            // println!("Twitch message: {}", twitch_message);
            // if result > 0 {
            //     println!("Bytes received: {}", result);
            //     println!("Message: {}", twitch_message);
            //
            // }
            match result {
                Ok(n) => {
                    if n > 0 {
                        println!("Bytes received: {}", n);
                        println!("Message: {}", twitch_message);
                        return twitch_message.starts_with("PING");
                    }
                },
                Err(e) => println!("Didn't receive anything yet .... {}", e)
            }

            return false;
        }
    }

    pub fn send_socket_message(mut writer: &TcpStream, msg: String) {
        let error_msg = format!("Error writing: {}", msg);
        writer.write_all(msg.as_bytes()).expect(error_msg.as_str());
        // write!(final_writer, "{}\r\n", msg).expect(error_msg.as_str());
        // println!("final writer len: {}", final_writer.buffer().len());
        writer.flush().expect("Error flushing stream writer");

        println!("{}", msg);
    }

    pub fn pong(mut writer: &TcpStream, msg: String) {

        let pong_msg = String::from("PONG");
        let final_pong_msg = format!("{} {}", pong_msg, msg);
        let error_msg = format!("Error writing: {}", pong_msg);
        writer.write_all(pong_msg.as_bytes()).expect(error_msg.as_str());
        writer.flush().expect("Error flushing PONG stream writer");
    }

    pub fn send_chuck_norris_fact(mut stream: &TcpStream) {
        // https://api.chucknorris.io/jokes/random

        let cn_uri = String::from("https://api.chucknorris.io/jokes/random".to_string());
        let client = reqwest::blocking::Client::new();

        let res = client.get(cn_uri)
            .send();

        let text_resp = match res {
            Ok(r) => {

                println!("random CN joke status, {:?}", r.status());
                // println!("oauth token body, {:?}", r.text());
                let r_text_string = r.text_with_charset("utf-8");
                Self::send_socket_message(stream, r_text_string.unwrap());
                // return r_text_string.unwrap()
                // "OK".to_string()
                // r.text().unwrap()
            },
            Err(e) => {
                println!("{}", e.to_string()) }
        };

    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn send_socket_message() {
        let listener_stream = TcpListener::bind("127.0.0.1:8080").expect("Error starting listener");
        let mut writer_stream = TcpStream::connect("127.0.0.1:8080").expect("Cannot connect to listener");

        for stream in listener_stream.incoming() {
            let mut stream = stream.unwrap();
            let mut stream_buf = String::new();

            stream.read_to_string(&mut stream_buf).expect("Error reading into buffer");

            assert_eq!(stream_buf.len(), 10);
        }

        writer_stream.write_all(b"Hello All!").expect("Error writing to writer stream");
    }
}
