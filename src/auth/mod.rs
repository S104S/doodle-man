extern crate reqwest;
extern crate serde;

use base64::{Engine as _, engine::{self, general_purpose}, alphabet};

use serde::{Serialize, Deserialize};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::format;
use std::io;
use std::io::{BufRead, BufWriter, Read, Write};
use std::time::Duration;
use std::net::*;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::IpAddr;
use std::net::UdpSocket;
use rand;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand::Rng;
use reqwest::Client;
use reqwest::header::{CONTENT_TYPE, CONTENT_LENGTH, HeaderValue, HeaderMap};

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
    pub host: String,
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
        Ok(oauth_token_message)
    }

    pub fn twitch_tcp_conn(twitch_tcp_conn_info: TwitchTcpConnection) -> Result<(), std::io::Error> {
        println!("Inside twitch_tcp_conn....");
        let token = twitch_tcp_conn_info.access_token;
        let host = twitch_tcp_conn_info.host;
        let port = twitch_tcp_conn_info.port;
        let final_uri = format!("{}:{}", host, port);

        println!("Final Socket URI: {}", final_uri);

        // connecting to Twitch with a TcpStream
        let twitch_stream = match TcpStream::connect(final_uri){
            Ok(stream) => stream,
            Err(E) => {
                println!("Error connecting to Socket Server: {}\r\n", E);
                 return Err(E);
            }
        };
            println!("Connected...");

        let mut reader = io::BufReader::new(twitch_stream.try_clone().expect("Failed to clone TCP stream"));
        let mut writer = io::BufWriter::new(twitch_stream);

        let mut final_writer = Self::send_http_socket_headers(&mut writer, host);
        println!("Socket headers len: {}", final_writer.buffer().len());
        final_writer.flush().expect("Error flushing headers");

        let tcp_auth_cap_req = "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands".to_string();
        let tcp_auth_pass = format!("PASS oauth:{}", token);
        let tcp_auth_nickname = "NICK broncosownersbox".to_string();
        let tcp_auth_username = "USER broncosownersbox".to_string();
        let tcp_auth_channel = "JOIN #pitcherlist".to_string();

        println!("Twitch Token: {}", tcp_auth_pass);

        Self::send_socket_message(&mut final_writer, tcp_auth_cap_req);
        Self::send_socket_message(&mut final_writer, tcp_auth_pass);
        Self::send_socket_message(&mut final_writer, tcp_auth_nickname);
        Self::send_socket_message(&mut final_writer, tcp_auth_channel);
        // let mut upgrade_connection_writer = Self::send_http_socket_headers(writer, host);
        // upgrade_connection_writer.flush().expect("Error flushing upgrade headers");

            loop {
                // println!("Inside loop...");
                let mut twitch_message: String = "".to_string();
                let result = reader.read_line(&mut twitch_message);

                match result {
                    Ok(n) => {
                        if n > 0 {
                            println!("Bytes received: {}", n);
                            println!("Message: {}", twitch_message);
                        }
                    },
                    Err(E) => println!("Didn't receive anything yet .... {}", E)
                }
            }
        // } else {
        //     println!("Not connected to socket server...");
        // }
        Ok(())
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
        writeln!(writer, "GET / HTTP/1.1").expect("GET: Error writing to writer");
        writer.flush().expect("Error flushing headers");
        writeln!(writer, "Accept-Encoding: gzip, deflate, br").expect("Accept-Encoding: Error writing to writer");
        writer.flush().expect("Error flushing headers");
        writeln!(writer, "Accept-Language: en-US,en;q=0.9").expect("Accept-Language: Error writing to writer");
        writer.flush().expect("Error flushing headers");
        writeln!(writer, "Cache-Control: no-cache").expect("Cache: Error writing to writer");
        writer.flush().expect("Error flushing headers");
        writeln!(writer, "Host: {}", host).expect("Host: Error writing to writer");
        writer.flush().expect("Error flushing headers");
        writeln!(writer, "Connection: Upgrade").expect("Connection: Error writing to writer");
        writer.flush().expect("Error flushing headers");
        writeln!(writer, "Upgrade: WebSocket").expect("Upgrade: Error writing to writer");
        writer.flush().expect("Error flushing headers");
        writeln!(writer, "Origin: https://www.twitch.tv").expect("Upgrade: Error writing to writer");
        writer.flush().expect("Error flushing headers");
        writeln!(writer, "Sec-WebSocket-Protocol: irc").expect("Sec-WebSocket-Protocol: Error writing to writer");
        writer.flush().expect("Error flushing headers");
        writeln!(writer, "Sec-WebSocket-Version: 13").expect("Sec-WebSocket-Version: Error writing to writer");
        writer.flush().expect("Error flushing headers");
        writeln!(writer, "Sec-WebSocket-Key: {}", final_b64).expect("Sec-WebSocket-Key: Error writing to writer");
        writer.flush().expect("Error flushing headers");
        // writer.flush().expect("Error flushing headers");
        // Flushing headers to the stream buffer
        writer
    }

    fn auth_with_socket_server(writer: BufWriter<TcpStream>, host: String, with_headers: bool) {
        // let mut final_writer = writer;



        if with_headers == true {

        }



        // writeln!(final_writer, "GET / HTTP/1.1").expect("GET: Error writing to writer");
        // writeln!(final_writer, "Accept-Encoding: gzip, deflate, br").expect("Accept-Encoding: Error writing to writer");
        // writeln!(final_writer, "Accept-Language: en-US,en;q=0.9").expect("Accept-Language: Error writing to writer");
        // writeln!(final_writer, "Cache-Control: no-cache").expect("Cache: Error writing to writer");
        // // writeln!(final_writer, "Host: {}", &host).expect("Host: Error writing to writer");
        // writeln!(final_writer, "Connection: Upgrade").expect("Connection: Error writing to writer");
        // writeln!(final_writer, "Upgrade: WebSocket").expect("Upgrade: Error writing to writer");
        // writeln!(final_writer, "Origin: https://www.twitch.tv").expect("Upgrade: Error writing to writer");
        // writeln!(final_writer, "{}", tcp_auth_cap_req).unwrap();
        // // writer.flush().expect("Error flushing stream writer");
        // writeln!(final_writer,  "{}",tcp_auth_pass).unwrap();
        // // writer.flush().expect("Error flushing stream writer");
        // writeln!(final_writer,  "{}",tcp_auth_username).unwrap();
        // writeln!(final_writer,  "{}",tcp_auth_nickname).unwrap();
        // println!("Second buffer len: {}", final_writer.buffer().len());
        // final_writer.flush().expect("Error flushing stream writer");
        //
        // writeln!(final_writer,  "{}",tcp_auth_channel).unwrap();
        // println!("Third buffer len: {}", final_writer.buffer().len());
        // final_writer.flush().expect("Error flushing stream writer");
        //
        // println!("writer flushed");
    }

    fn send_socket_message(writer: &mut BufWriter<TcpStream>, msg: String) {
        let mut final_writer = writer;
        writeln!(final_writer, "{}", msg).unwrap();
        final_writer.flush().expect("Error flushing stream writer");

        println!("{} written and flushed.", msg);
    }
}