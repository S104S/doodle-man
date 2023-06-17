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

        let tcp_auth_cap_req = "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands".to_string();
        let tcp_auth_pass = format!("PASS oauth:{}", token);
        let tcp_auth_nickname = "NICK broncosownersbox".to_string();
        let tcp_auth_username = "USER broncosownersbox".to_string();
        let tcp_auth_channel = "JOIN #denverbroncoscom".to_string();


        println!("Twitch Token: {}", tcp_auth_pass);
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
        let mut rng = rand::thread_rng();
        let random_bytes: Vec<u8> = (0..16).map(|_| rng.gen()).collect();
        const CUSTOM_ENGINE: engine::GeneralPurpose =
            engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);
        // Convert bytes to base64 string
        let mut buf = String::new();
        CUSTOM_ENGINE.encode_string(&random_bytes, &mut buf);
        // let b64 = general_purpose::STANDARD.encode(b"hello world~");
        println!("b64 key: {}", buf);
        let final_b64 = format!("{}=", buf);
        writeln!(&mut writer, "GET / HTTP/1.1\r\n").unwrap();
        // writer.flush().expect("Error flushing stream writer");
        writeln!(&mut writer, "Upgrade:websocket\r\n").unwrap();
        // writer.flush().expect("Error flushing stream writer");
        writeln!(&mut writer, "Connection:Upgrade\r\n").unwrap();
        // writer.flush().expect("Error flushing stream writer");
        writeln!(&mut writer, "Host:{}\r\n", host).unwrap();
        // writer.flush().expect("Error flushing stream writer");
        writeln!(&mut writer, "Origin:https://www.twitch.tv\r\n").unwrap();
        // writer.flush().expect("Error flushing stream writer");
        writeln!(&mut writer, "Sec-WebSocket-Protocol:irc\r\n").unwrap();
        // writer.flush().expect("Error flushing stream writer");
        writeln!(&mut writer, "Sec-WebSocket-Version:13\r\n").unwrap();
        // writer.flush().expect("Error flushing stream writer");
        writeln!(&mut writer, "Sec-WebSocket-Key:{}\r\n", final_b64).unwrap();
        // writer.flush().expect("Error flushing stream writer");

        writeln!(&mut writer, "{}\r\n", tcp_auth_cap_req).unwrap();
        // writer.flush().expect("Error flushing stream writer");
        writeln!(&mut writer,  "{}\r\n",tcp_auth_pass).unwrap();
        // writer.flush().expect("Error flushing stream writer");
        writeln!(&mut writer,  "{}\r\n",tcp_auth_nickname).unwrap();
        writer.flush().expect("Error flushing stream writer");

        // writeln!(&mut writer,  "JOIN {}\r\n",tcp_auth_channel).expect("Error writing JOIN");
        // writer.flush().expect("Error flushing stream writer");
            // twitch_stream.write(&[1]).expect("Error writing buffer");
            // twitch_stream.read(&mut [0; 128]).expect("Error reading message");

            // twitch_stream.write(tcp_auth_cap_req.as_bytes()).expect("Error sending auth cap req packet.");
            // twitch_stream.write(tcp_auth_cap_req.as_bytes()).unwrap();

            // // twitch_stream.flush().expect("Error flushing cap req.");
            // twitch_stream.write(tcp_auth_msg.as_bytes()).unwrap();
            // twitch_stream.flush().unwrap();
            // twitch_stream.write(tcp_auth_msg.as_bytes()).expect("Error sending auth token packet.");
            // twitch_stream.flush().expect("Error flushing auth.");
            // twitch_stream.write(tcp_auth_nickname.as_bytes()).unwrap();
            // twitch_stream.flush().unwrap();
            // twitch_stream.write(tcp_auth_username.as_bytes()).unwrap();
            // twitch_stream.flush().unwrap();
            // twitch_stream.write(tcp_auth_channel.as_bytes()).unwrap();
            // twitch_stream.flush().unwrap();
            // // twitch_stream.write(tcp_auth_username.as_bytes()).expect("Error sending auth username packet.");
            // // twitch_stream.flush().expect("Error flushing username.");
            //
            // println!("Auth packages sent...");
            // twitch_stream.shutdown(Shutdown::Both)
            //     .expect("Error shutting down Twitch stream");
            // twitch_stream.flush().unwrap();
            // // data_flushed.unwrap();
            //
            loop {
                // println!("Inside loop...");
                let mut twitch_message: String = "".to_string();
                let result = reader.read_to_string(&mut twitch_message);

                match result {
                    Ok(n) => {
                        if n > 0 {
                            println!("Bytes received: {}", n);
                            println!("Message: {}", twitch_message);
                        }
                    },
                    _ => println!("Didn't receive anything yet ....")
                }
            }
        // } else {
        //     println!("Not connected to socket server...");
        // }
        Ok(())
    }
}