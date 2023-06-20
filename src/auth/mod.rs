extern crate reqwest;
extern crate serde;

use base64::{Engine as _, engine::{self, general_purpose}, alphabet};

use serde::{Serialize, Deserialize};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::format;
use std::{io, thread};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::time::Duration;
use std::net::*;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::IpAddr;
use std::net::UdpSocket;
use std::sync::{Arc, Mutex};
use rand;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, CONTENT_LENGTH, HeaderValue, HeaderMap, AUTHORIZATION};

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
    pub host: [SocketAddr; 3],
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

    pub async fn twitch_tcp_conn(twitch_tcp_conn_info: TwitchTcpConnection) {
        println!("Inside twitch_tcp_conn....");
        println!("twitch tcp conn: {}", twitch_tcp_conn_info.access_token);
        let token = twitch_tcp_conn_info.access_token;
        let host = twitch_tcp_conn_info.host;
        let port = twitch_tcp_conn_info.port;
        // let final_uri = format!("{}:{}", host, port);

        // println!("Final Socket URI: {}", final_uri);

        let mut streamer = TcpStream::connect(&twitch_tcp_conn_info.host[0]).await.unwrap();

        // connecting to Twitch with a TcpStream
        if let Ok(twitch_stream) = TcpStream::connect(&twitch_tcp_conn_info.host[..]) {
            println!("Connected...");

            let host_ip = &twitch_stream.peer_addr().unwrap().ip();
            let host_port = &twitch_stream.peer_addr().unwrap().port();

            let mut reader = Arc::new(
                Mutex::new(
                    io::BufReader::new(
                        twitch_stream.try_clone().expect("Failed to clone TCP stream"))));
            let mut writer = io::BufWriter::new(twitch_stream);

            thread::spawn(move || {
                Self::handle_incoming_messages(reader);
            });

            let final_host_uri = format!("{}:{}", host_ip, host_port);
            println!("final host uri: {}", final_host_uri);
            // let mut final_writer= writer;
            let mut final_writer = Self::send_http_socket_headers(&mut writer,final_host_uri);
            println!("Socket headers len: {}", final_writer.buffer().len());
            // final_writer.flush().expect("Error flushing headers");

            let twitch_username = "broncosownersbox".to_string();
            let channel = "pitcherlist".to_string();

            let tcp_auth_cap_req = "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands".to_string();
            let tcp_auth_pass = format!("PASS oauth:{}", token);
            let tcp_auth_nickname = format!("NICK {}", twitch_username);
            let tcp_auth_username = format!("USER {}", twitch_username);
            let tcp_auth_channel = format!("JOIN #{}", channel);

            // println!("Twitch Token: {}", tcp_auth_pass);

            // Self::send_socket_message(&mut final_writer, tcp_auth_cap_req);
            Self::send_socket_message(&mut final_writer, tcp_auth_pass);
            Self::send_socket_message(&mut final_writer, tcp_auth_nickname);
            // Self::send_socket_message(&mut final_writer, tcp_auth_username);
            // Self::send_socket_message(&mut final_writer, tcp_auth_channel);
            // Ok(())
        } else {
            println!("Error connecting to socket server...");
        }

    }

    fn create_headers(host: String, websocket_key: String) -> Vec<String> {
        let mut headers: Vec<String> = Vec::new();
        let hard_coded_host = "irc-ws.chat.twitch.tv".to_string();
        headers.push(String::from("GET / HTTP/1.1"));
        // headers.push(String::from("Accept-Encoding:gzip, deflate, br"));
        // headers.push(String::from("Accept-Language:en-US,en;q=0.9"));
        // headers.push(String::from("Cache-Control:no-cache"));
        headers.push(String::from("Connection:Upgrade"));
        headers.push(String::from("Upgrade:websocket"));
        // headers.push(format!("Host:{}", host));
        headers.push(format!("Host:{}", hard_coded_host));
        headers.push(String::from("Origin:https://www.twitch.tv"));
        headers.push(String::from("Sec-WebSocket-Protocol:irc"));
        headers.push(String::from("Sec-WebSocket-Version:13"));
        headers.push(format!("Sec-WebSocket-Key:{}", websocket_key));
        headers
    }

    fn send_http_socket_headers(writer: &mut BufWriter<TcpStream>, host: String) -> &mut BufWriter<TcpStream> {
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        const CUSTOM_ENGINE: engine::GeneralPurpose =
            engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::PAD);
        // Convert bytes to base64 string
        let mut buf = String::new();
        CUSTOM_ENGINE.encode_string(&random_bytes, &mut buf);
        println!("b64 key: {}", buf);
        let final_b64 = format!("{}", buf);
        println!("final b64: {}", final_b64);
        let headers = Self::create_headers(host, final_b64);

        for header in headers.iter() {
            // println!("Writing header: {}", header);
            let error_msg = format!("Error writing header: {}", header);
            write!(writer, "{}\r\n", header).expect(error_msg.as_str());
            writer.flush().expect("Error flushing headers");
        }
        writer
    }

    fn handle_incoming_messages(mut stream: Arc<Mutex<BufReader<TcpStream>>>) {
        loop {
            // println!("Inside loop...");
            let mut twitch_message: String = String::new();
            let mut tcp_stream = stream.lock().unwrap();
            let result = tcp_stream.read_to_string(&mut twitch_message);

            // println!("Twitch message: {}", twitch_message);
            match result {
                Ok(n) => {
                    if n > 0 {
                        println!("Bytes received: {}", n);
                        println!("Message: {}", twitch_message);
                    }
                },
                Err(e) => println!("Didn't receive anything yet .... {}", e)
            }
        }
    }

    fn send_socket_message(writer: &mut BufWriter<TcpStream>, msg: String) {
        let mut final_writer = writer;
        writeln!(final_writer, "{}", msg).unwrap();
        println!("final writer len: {}", final_writer.buffer().len());
        final_writer.flush().expect("Error flushing stream writer");

        println!("{} written and flushed.", msg);
    }
}