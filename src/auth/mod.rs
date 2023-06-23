extern crate reqwest;
extern crate serde;

use base64::{Engine as _, engine::{self, general_purpose}, alphabet};

use serde::{Serialize, Deserialize};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::format;
use std::{io, thread, time};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::time::Duration;
use std::net::*;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::IpAddr;
use std::net::UdpSocket;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use rand;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, CONTENT_LENGTH, HeaderValue, HeaderMap, AUTHORIZATION};
use crate::messages;

use messages::Message;

pub struct Auth {
   pub key: String
}

#[derive(Serialize, Deserialize)]
pub struct TokenMessage {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i32,
    pub token_type: String
}

pub struct TwitchAuthInfo {
    pub uri: String,
    pub client_id: String,
    pub client_secret: String,
    pub code: String,
    pub redirect_uri: String,
    pub grant_type: String
}

pub struct TwitchTcpConnection {
    pub access_token: String,
    pub host: [SocketAddr; 4],
    pub port: i16,
}

impl Auth {
    pub fn twitch(auth_info: TwitchAuthInfo) -> std::result::Result<TokenMessage, &'static str> {
        println!("Authenticating with Twitch....");
        println!("Service Auth URL: {}", auth_info.uri);
        // [("client_id", auth_info.client_id),
        //     ("client_secret", auth_info.client_secret),
        //     ("code", auth_info.code),
        //     ("grant_type", auth_info.grant_type), ("redirect_uri", auth_info.redirect_uri)];
        let mut params = HashMap::new();
        params.insert("client_id", auth_info.client_id);
        params.insert("client_secret", auth_info.client_secret);
        params.insert("code", auth_info.code);
        params.insert("grant_type", auth_info.grant_type);
        params.insert("redirect_uri", auth_info.redirect_uri);

        println!("{:?}", params);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));

        let client = reqwest::blocking::Client::new();

        let res = client.post(auth_info.uri)
            .headers(headers)
            .form(&params)
            .send();

        let text_resp = match res {
            Ok(r) => {

                println!("oauth token status, {:?}", r.status());
                // println!("oauth token body, {:?}", r.text());
                let r_text_string = r.text_with_charset("utf-8");
                r_text_string.unwrap()
                // "OK".to_string()
                // r.text().unwrap()
            },
            Err(e) => {
                // println!("error getting auth token: {e:?}");
                return Err("Error getting auth token")
            }
        };

        println!("Auth resp: {:?}", text_resp.to_string());

        let oauth_token_message: TokenMessage = serde_json::from_str(&text_resp).unwrap();

        let access_token_to_validate = format!("OAuth {}", &oauth_token_message.access_token);
        println!("Access token to valide: {}", access_token_to_validate);
        let mut validation_headers = HeaderMap::new();
        validation_headers.insert(AUTHORIZATION, HeaderValue::from_str(access_token_to_validate.as_str()).unwrap());

        let token_validation_resp =
            client.get("https://id.twitch.tv/oauth2/validate".to_string())
                    .headers(validation_headers)
                    .send();

        println!("TVR: {:?}", token_validation_resp.unwrap().text());
        // match token_validation_resp {
        //     Ok(r) => {
        //         println!("Validation Resp: {:?}", r.text());
        //     },
        //     Err(e) => {
        //         return Err("Error validating token")
        //     }
        // }

        Ok(oauth_token_message)
    }

    pub fn twitch_tcp_conn(twitch_tcp_conn_info: TwitchTcpConnection) {
        println!("Inside twitch_tcp_conn....");
        println!("twitch tcp conn: {}", twitch_tcp_conn_info.access_token);
        let token = twitch_tcp_conn_info.access_token;
        // let host = twitch_tcp_conn_info.host;
        // let port = twitch_tcp_conn_info.port;

        // connecting to Twitch with a TcpStream
        let mut twitch_stream = TcpStream::connect(&twitch_tcp_conn_info.host[..]).unwrap();
        // let mut twitch_stream = TcpStream::connect(  SocketAddr::from(([44,237,40,50], 6667))).unwrap();
        // let mut twitch_stream = TcpStream::connect( "irc.twitch.tv:80").unwrap();
        // let mut buf = String::new();
        // twitch_stream.read_to_string(&mut buf).unwrap();
        // println!("buf: {}", buf);
        let reader_stream = twitch_stream.try_clone().unwrap();
        let writer_stream = twitch_stream.try_clone().unwrap();
        // if let Ok(twitch_stream) = TcpStream::connect(&twitch_tcp_conn_info.host[..]) {

            println!("Connected...");
            // let mut reader = Arc::new(
            //
            //         io::BufReader::new(
            //             twitch_stream.try_clone().expect("Failed to clone TCP stream")));
            // let mut writer = Arc::new(Mutex::new(io::BufWriter::new(twitch_stream)));

            // let messages_reader_builder = thread::Builder::new();
            let join_reader_handle = thread::spawn(move || {
                // let mut reader = io::BufReader::new(&twitch_stream);
                Message::handle_incoming_messages(&reader_stream);
            });

            // let message_writer_builder = thread::Builder::new();
            // let join_writer_handler = thread::spawn(move || {
                // let mut tcp_write_stream = writer.lock().unwrap();


                // println!("final host uri: {}", final_host_uri);
                let sleep_count = 5000;

                // let mut writer = io::BufWriter::new(&twitch_stream);
                // let mut final_stream = &writer_stream;
                let mut final_stream = Self::send_http_socket_headers(&writer_stream);

                // println!("Socket headers len: {}", final_writer.buffer().len());
                // final_writer.flush().expect("Error flushing headers");

                let chat_bot_name = "doodleman".to_string();
                let twitch_username = "broncosownersbox".to_string();
                let channel = "betql".to_string();

                let tcp_auth_cap_req_membership = "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands\r".to_string();
                let tcp_auth_cap_req_tags = "CAP REQ :twitch.tv/tags\r".to_string();
                let tcp_auth_cap_req_commands = "CAP REQ :twitch.tv/commands\r".to_string();
                let tcp_auth_pass = format!("PASS oauth:{} \r", token.as_str());
                let tcp_auth_nickname = format!("NICK {} \r", twitch_username.as_str());
                let tcp_auth_username = format!("USER {} 8 *:{}", twitch_username, twitch_username);
                let tcp_auth_channel = format!("JOIN #{}", channel);

                // println!("Twitch Token: {}", tcp_auth_pass);

                let ten_millis = time::Duration::from_millis(sleep_count);

                Message::send_socket_message(&final_stream, tcp_auth_pass.trim().to_string());
                Message::send_socket_message(&final_stream, tcp_auth_nickname.trim().to_string());
                //Message::send_chuck_norris_fact(&final_stream);
                // println!("CN Fact: {}", random_fact);
                // Message::send_socket_message(&mut final_writer, tcp_auth_username);
                // Message::send_socket_message(&final_stream, tcp_auth_cap_req_membership);
                // Message::send_socket_message(&final_stream, tcp_auth_cap_req_tags);
                // Message::send_socket_message(&final_stream, tcp_auth_cap_req_commands);

                // thread::sleep(ten_millis);
                // Message::send_socket_message(&final_stream, tcp_auth_channel.trim().to_string());
            // });

            join_reader_handle.join().expect("Couldn't join the incoming messages thread");
            // join_writer_handler.join().expect("Couldn't join the messages writer thread");
        // } else {
        //     println!("Error connecting to socket server...");
        // }

    }

    fn create_headers(host: String, websocket_key: String) -> Vec<String> {
        let mut headers: Vec<String> = Vec::new();
        let hard_coded_host = "irc.twitch.tv".to_string();
        headers.push(String::from("GET / HTTP/1.1\r"));
        // headers.push(String::from("Accept-Encoding:gzip, deflate, br"));
        // headers.push(String::from("Accept-Language:en-US,en;q=0.9"));
        // headers.push(String::from("Cache-Control:no-cache"));
        headers.push(String::from("Connection: Upgrade\r"));
        headers.push(String::from("Upgrade: websocket\r"));
        // headers.push(format!("Host:{}", host));
        headers.push(format!("Host: {}\r", hard_coded_host));
        headers.push(String::from("Origin: https://www.twitch.tv\r"));
        headers.push(String::from("Sec-WebSocket-Protocol: irc\r"));
        headers.push(String::from("Sec-WebSocket-Version: 13\r"));
        headers.push(format!("Sec-WebSocket-Key: {}\r", websocket_key));
        headers
    }

    fn send_http_socket_headers(mut writer: &TcpStream) -> &TcpStream {
        let mut buf = String::new();
        let mut rng = rand::thread_rng();
        let host_ip = writer.peer_addr().unwrap().ip();
        let host_port = writer.peer_addr().unwrap().port();
        let final_host_uri = format!("{}:{}", host_ip, host_port);
        let random_bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        const CUSTOM_ENGINE: engine::GeneralPurpose =
            engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::PAD);
        CUSTOM_ENGINE.encode_string(&random_bytes, &mut buf);

        println!("b64 key: {}", buf);
        let final_b64 = format!("{}", buf);
        println!("final b64: {}", final_b64);
        let headers = Self::create_headers(final_host_uri, final_b64);

        for header in headers.iter() {
            println!("Writing header: {}", header);

            let error_msg = format!("Error writing header: {}", header);
            // write!(writer, "{}", header).expect(error_msg.as_str());
            writer.write_all(header.as_bytes()).expect(error_msg.as_str());
            writer.flush().expect("Error flushing headers");
        }
        writer
    }
}