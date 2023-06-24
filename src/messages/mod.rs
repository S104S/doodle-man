use io::BufReader;
use reqwest::{Client, Url};
use std::alloc::handle_alloc_error;
use std::borrow::Cow;
use std::convert::TryInto;
use std::io::{Read, Write};
use std::net::{SocketAddrV6, TcpListener, TcpStream, ToSocketAddrs};
use std::{io, thread};

pub struct Message {}

impl Message {
    pub fn handle_incoming_messages(mut stream: &TcpStream, channel: String, token: String) {
        println!("Listening for messages....");
        loop {
            let mut twitch_message: Vec<u8> = vec![];
            let result = stream.read_to_end(&mut twitch_message);
            match result {
                Ok(n) => {
                    // println!("N: {}", n);
                    if n > 0 {
                        println!("Bytes received: {}", n);
                        let msg = String::from_utf8_lossy(&*twitch_message);
                        println!("Message: {:?}", msg);

                        Self::process_incoming_message(stream, msg, &channel);
                    }
                }
                Err(e) => println!("Error in stream read result .... {}", e),
            }
        }
    }

    fn process_incoming_message(mut stream: &TcpStream, msg: Cow<str>, channel: &String) {
        let msg_split: Vec<&str> = msg.split(" ").collect();
        let command = msg_split[1].to_string();

        match command.as_str() {
            "PING" => {
                Self::pong(stream, msg_split);
            }
            "PRIVMSG" => {
                Self::send_chuck_norris_fact(stream, &channel);
            }
            _ => println!("No matching command sent"),
        }
    }

    pub fn send_socket_message(mut writer: &TcpStream, msg: String) {
        let error_msg = format!("Error writing: {}", msg);
        println!("Message being sent: {}", msg);
        writer.write_all(msg.as_bytes()).expect(error_msg.as_str());
        writer.flush().expect("Error flushing stream writer");
    }

    pub fn pong(mut writer: &TcpStream, msg: Vec<&str>) {
        let pong_msg = String::from("PONG");
        let final_pong_msg = format!("{} {}", pong_msg, msg[0].to_string());
        let error_msg = format!("Error writing: {}", pong_msg);
        writer
            .write_all(pong_msg.as_bytes())
            .expect(error_msg.as_str());
        writer.flush().expect("Error flushing PONG stream writer");
    }

    pub fn send_chuck_norris_fact(mut stream: &TcpStream, channel: &String) {
        let cn_uri = String::from("https://api.chucknorris.io/jokes/random".to_string());
        let client = reqwest::blocking::Client::new();

        let res = client.get(cn_uri).send();

        let text_resp = match res {
            Ok(r) => {
                let r_text_string = r.text_with_charset("utf-8");
                let cn_msg = format!("{} #{} :{}", "PRIVMSG", channel, r_text_string.unwrap());
                Self::send_socket_message(stream, cn_msg);
            }
            Err(e) => {
                println!("{}", e.to_string())
            }
        };
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // #[test]
    // fn send_socket_message() {
    //     let listener_stream = TcpListener::bind("127.0.0.1:8080").expect("Error starting listener");
    //     let mut writer_stream = TcpStream::connect("127.0.0.1:8080").expect("Cannot connect to listener");
    //
    //     for stream in listener_stream.incoming() {
    //         let mut stream = stream.unwrap();
    //         let mut stream_buf = String::new();
    //
    //         stream.read_to_string(&mut stream_buf).expect("Error reading into buffer");
    //
    //         assert_eq!(stream_buf.len(), 10);
    //     }
    //
    //     writer_stream.write_all(b"Hello All!").expect("Error writing to writer stream");
    // }

    // #[test]
    // fn process_incoming_message() {
    //     let msg = String::from(":foo!foo@foo.tmi.twitch.tv PRIVMSG #bar :bleedPurple");
    //
    //     let command = Message::process_incoming_message(msg);
    //     println!("{}", command);
    //
    //     assert_eq!(command, "PRIVMSG");
    // }
}
