#![feature(addr_parse_ascii)]

extern crate core;

use std::{io, thread};

mod auth;
mod messages;
mod settings;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::process::{Command, Stdio};
use std::str;

fn handle_connection(mut stream: TcpStream) -> String {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";

    if buffer.starts_with(get) {
        let mut file = File::open("auth.html").unwrap();
        let mut contents = String::new();

        file.read_to_string(&mut contents).unwrap();

        let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents);

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    } else {
        let buffer_str =
            std::str::from_utf8(&buffer[0..100]).expect("Error getting part of buffer.");
        let access_token_split: Vec<&str> = buffer_str.split("=").collect();
        let access_token: Vec<&str> = access_token_split[1].split("&").collect();

        return access_token[0].to_string();
    }

    "No access token".to_string()
}
fn main() {
    println!("OS being used: {}", env::consts::OS);
    let current_os = env::consts::OS;
    let settings = settings::Settings::new().unwrap();
    let twitch_config = settings.twitch_config;
    let channel_info = settings.twitch_channel_info;
    let mut codes_used: Vec<String> = Vec::new();
    let twitch_user_auth_uri = twitch_config.user_auth_uri;
    let client_id = twitch_config.client_id;
    let response_type = twitch_config.response_type;
    let redirect_uri = twitch_config.redirect_uri;
    let final_user_auth_uri = format!(
        "{}?client_id={}&response_type={}&redirect_uri={}&scope=chat:read+chat:edit",
        twitch_user_auth_uri, client_id, response_type, redirect_uri
    );

    let mut default_browser_program = String::new();

    match current_os {
        "windows" => {
            default_browser_program = String::from("rundll32");
        }
        "darwin" => {
            default_browser_program = String::from("open");
        }
        "linux" => {
            default_browser_program = String::from("xdg-open");
        }
        _ => println!("Browser not supported."),
    }

    let _ = Command::new(&default_browser_program)
        .arg(final_user_auth_uri)
        .stdout(Stdio::null())
        .spawn()
        .expect("Error opening the Twitch User auth URL.");

    let web_server = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in web_server.incoming() {
        let token = handle_connection(stream.unwrap());

        if token != "No access token" {
            let twitch_service_authorize_uri = &twitch_config.oauth_uri;
            let twitch_socket_server = &twitch_config.socket_server;
            let ssl_port = 443;
            let mut token_message = auth::TokenMessage {
                access_token: "".to_string(),
                refresh_token: "".to_string(),
                expires_in: 0,
                token_type: "".to_string(),
            };
            if !codes_used.iter().any(|r| r == &token) {
                codes_used.push(token.clone());

                let secret = &twitch_config.secret;
                let grant_type = &twitch_config.grant_type;
                let final_service_auth_uri = format!("{}", twitch_service_authorize_uri);

                let auth_info = auth::TwitchAuthInfo {
                    uri: final_service_auth_uri,
                    client_id: client_id.to_string(),
                    client_secret: secret.to_string(),
                    code: token.to_string(),
                    grant_type: grant_type.to_string(),
                    redirect_uri: redirect_uri.to_string(),
                };

                let is_authed = auth::Auth::twitch(auth_info);
                token_message = is_authed.unwrap();

                let twitch_tcp_conn_info = auth::TwitchTcpConnection {
                    access_token: token_message.access_token,
                    host: twitch_socket_server.to_string(),
                    port: ssl_port,
                    channel: channel_info.channel.to_string(),
                    chat_bot_name: channel_info.chat_bot_name.to_string(),
                    username: channel_info.username.to_string(),
                };

                auth::Auth::twitch_tcp_conn(twitch_tcp_conn_info);
            }
        }
    }
}
