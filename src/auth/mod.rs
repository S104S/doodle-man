extern crate reqwest;
extern crate serde;

use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine as _,
};

use crate::messages;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use rand;
use rand::Rng;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_LENGTH, CONTENT_TYPE};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::format;
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::net::IpAddr;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::net::TcpStream;
use std::net::UdpSocket;
use std::net::*;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{io, thread, time};

use messages::Message;

pub struct Auth {
    pub key: String,
}

#[derive(Serialize, Deserialize)]
pub struct TokenMessage {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i32,
    pub token_type: String,
}

pub struct TwitchAuthInfo {
    pub uri: String,
    pub client_id: String,
    pub client_secret: String,
    pub code: String,
    pub redirect_uri: String,
    pub grant_type: String,
}

pub struct TwitchTcpConnection {
    pub access_token: String,
    pub host: String,
    pub port: i16,
    pub channel: String,
    pub chat_bot_name: String,
    pub username: String,
}

impl Auth {
    pub fn twitch(auth_info: TwitchAuthInfo) -> std::result::Result<TokenMessage, &'static str> {
        let mut params = HashMap::new();
        params.insert("client_id", auth_info.client_id);
        params.insert("client_secret", auth_info.client_secret);
        params.insert("code", auth_info.code);
        params.insert("grant_type", auth_info.grant_type);
        params.insert("redirect_uri", auth_info.redirect_uri);

        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        let client = reqwest::blocking::Client::new();
        let res = client
            .post(auth_info.uri)
            .headers(headers)
            .form(&params)
            .send();

        let text_resp = match res {
            Ok(r) => {
                let r_text_string = r.text_with_charset("utf-8");
                r_text_string.unwrap()
            }
            Err(e) => return Err("Error getting auth token"),
        };

        let oauth_token_message: TokenMessage = serde_json::from_str(&text_resp).unwrap();
        let access_token_to_validate = format!("OAuth {}", &oauth_token_message.access_token);
        let mut validation_headers = HeaderMap::new();

        validation_headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(access_token_to_validate.as_str()).unwrap(),
        );

        let token_validation_resp = client
            .get("https://id.twitch.tv/oauth2/validate".to_string())
            .headers(validation_headers)
            .send();

        // println!("TVR: {:?}", token_validation_resp.unwrap().text());
        // match token_validation_resp {
        //     Ok(r) => r,
        //     Err(e) => {
        //         return Err("Error validating token")
        //     }
        // }

        Ok(oauth_token_message)
    }

    pub fn twitch_tcp_conn(twitch_tcp_conn_info: TwitchTcpConnection) {
        let token = twitch_tcp_conn_info.access_token;
        let chat_bot_name = twitch_tcp_conn_info.chat_bot_name;
        let twitch_username = twitch_tcp_conn_info.username;
        let channel = twitch_tcp_conn_info.channel;
        let line_break = "\r";
        let tcp_auth_cap_req_membership = format!(
            "CAP REQ :twitch.tv/membership twitch.tv/tags twitch.tv/commands{}",
            line_break
        );
        let tcp_auth_cap_req_tags = format!("CAP REQ :twitch.tv/tags{}", line_break);
        let tcp_auth_cap_req_commands = format!("CAP REQ :twitch.tv/commands{}", line_break);
        let tcp_auth_pass = format!("PASS oauth:{} {}", token, line_break);
        let tcp_auth_nickname = format!("NICK {} {}", chat_bot_name, line_break);
        let tcp_auth_username = format!(
            "USER {} 8 *:{}{}",
            twitch_username, twitch_username, line_break
        );
        let tcp_auth_channel = format!("JOIN #{}{}", &channel, line_break);

        let thread_channel = channel.clone();
        // connecting to Twitch with a TcpStream
        println!("Connecting....");
        let mut twitch_stream = TcpStream::connect(twitch_tcp_conn_info.host).unwrap();
        let writer_stream = twitch_stream.try_clone().unwrap();
        let reader_stream = twitch_stream.try_clone().unwrap();
        // upgrading to websockets
        println!("Upgrading to websockets....");
        let mut stream_with_headers = Self::send_http_socket_headers(&twitch_stream);
        stream_with_headers.flush().expect("Error flushing again.");
        let ten_millis = time::Duration::from_millis(3000);
        let now = time::Instant::now();

        println!("Sending auth messages....");
        Message::send_socket_message(
            &stream_with_headers,
            tcp_auth_cap_req_membership.trim().to_string(),
        );
        Message::send_socket_message(&stream_with_headers, tcp_auth_pass.trim().to_string());
        Message::send_socket_message(&stream_with_headers, tcp_auth_nickname.trim().to_string());
        thread::sleep(ten_millis);

        Message::send_socket_message(&stream_with_headers, tcp_auth_channel.trim().to_string());
        // Creating thread to listen to incoming messages from Twitch server
        let join_reader_handle = thread::spawn(move || {
            Message::handle_incoming_messages(&reader_stream, thread_channel, token);
        });

        join_reader_handle
            .join()
            .expect("Couldn't join the incoming messages thread");
    }

    fn create_headers(host: String, websocket_key: String) -> Vec<String> {
        let mut headers: Vec<String> = Vec::new();
        let line_break = "\n";
        headers.push(format!("GET / HTTP/1.1{}", line_break));
        headers.push(format!("Accept-Encoding:gzip, deflate, br{}", line_break));
        headers.push(format!("Accept-Language:en-US,en;q=0.9{}", line_break));
        headers.push(format!("Cache-Control:no-cache{}", line_break));
        headers.push(format!("Connection:Upgrade{}", line_break));
        headers.push(format!("Upgrade:websocket{}", line_break));
        headers.push(format!("Host:{}{}", host, line_break));
        headers.push(format!("Origin: https://www.twitch.tv{}", line_break));
        headers.push(format!("Sec-WebSocket-Protocol:irc{}", line_break));
        headers.push(format!("Sec-WebSocket-Version:13{}", line_break));
        headers.push(format!("Sec-WebSocket-Key:{}{}", websocket_key, line_break));
        headers.push(format!("{}", line_break));
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

        let final_b64 = format!("{}", buf);
        println!("{}", final_host_uri);
        let headers = Self::create_headers(final_host_uri, final_b64);

        for header in headers.iter() {
            let error_msg = format!("Error writing header: {}", header);
            writer
                .write_all(header.as_bytes())
                .expect(error_msg.as_str());
            writer.flush().expect("Error flushing headers");
        }
        writer
    }
}
