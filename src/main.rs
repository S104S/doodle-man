#![feature(addr_parse_ascii)]

use std::{io, thread};

mod auth;
mod messages;
mod options;
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
        // println!("Access Token Buffer: {}", buffer_str);
        // println!("Access Token Split: {}", access_token_split[0]);
        let access_token: Vec<&str> = access_token_split[1].split("&").collect();
        // println!("Final Access Token: {}", access_token[0]);
        // let buf_reader = BufReader::new(&mut stream);
        // let http_request: Vec<_> = buf_reader
        //     .lines()
        //     .map(|result| result.unwrap())
        //     .take_while(|line| !line.is_empty())
        //     .collect();

        // println!("Request: {:#?}", http_request);

        return access_token[0].to_string();
    }

    "No access token".to_string()
}
fn main() {
    println!("OS being used: {}", env::consts::OS);
    let mut codes_used: Vec<String> = Vec::new();
    let twitch_user_auth_uri = "https://id.twitch.tv/oauth2/authorize";
    let client_id = "6pudbm618q548pmpyea9aqkc4lsgj1";
    let response_type = "code";
    let redirect_uri = "http://localhost:8080";
    let final_user_auth_uri = format!(
        "{}?client_id={}&response_type={}&redirect_uri={}&scope=chat:read+chat:edit",
        twitch_user_auth_uri, client_id, response_type, redirect_uri
    );

    println!("User Auth URL: {}", final_user_auth_uri);
    let openurl = Command::new("xdg-open")
        .arg(final_user_auth_uri)
        .stdout(Stdio::null())
        .spawn()
        .expect("Error opening the Twitch User auth URL.");
    // println!("URL Status: {}", openurl.status);

    let web_server = TcpListener::bind("127.0.0.1:8080").unwrap();

    // web_server.set_nonblocking(true).expect("Error setting web server to non-blocking.");
    for stream in web_server.incoming() {
        // println!("Received something...");
        let token = handle_connection(stream.unwrap());

        if token != "No access token" {
            let twitch_service_authorize_uri = "https://id.twitch.tv/oauth2/token";
            let twitch_ssl_host_addrs = [
                SocketAddr::from(([34, 212, 92, 60], 6667)),
                SocketAddr::from(([54, 187, 159, 249], 80)),
                SocketAddr::from(([34, 211, 20, 86], 443)),
                SocketAddr::from(([34, 212, 92, 60], 6697)),
            ];
            let ssl_port = 443;
            let mut token_message = auth::TokenMessage {
                access_token: "".to_string(),
                refresh_token: "".to_string(),
                expires_in: 0,
                token_type: "".to_string(),
            };
            // println!("Token received: {}", token);
            if !codes_used.iter().any(|r| r == &token) {
                codes_used.push(token.clone());

                let secret = "";
                let grant_type = "authorization_code";
                let final_service_auth_uri = format!("{}", twitch_service_authorize_uri);

                let params = [
                    ("client_id", client_id),
                    ("client_secret", secret),
                    ("code", &token.to_string()),
                    ("grant_type", grant_type),
                    ("redirect_uri", redirect_uri),
                ];

                let auth_info = auth::TwitchAuthInfo {
                    uri: final_service_auth_uri,
                    client_id: client_id.to_string(),
                    client_secret: secret.to_string(),
                    code: token.to_string(),
                    grant_type: grant_type.to_string(),
                    redirect_uri: redirect_uri.to_string(),
                };

                // let final_service_auth_uri = format!("{}?client_id={}&client_secret={}&grant_type={}&redirect_uri={}",
                //                                      twitch_service_authorize_uri, client_id, secret, grant_type, redirect_uri);

                let is_authed = auth::Auth::twitch(auth_info);
                // let is_authed = auth::Auth::twitch_tcp(twitch_ssl_host, twitch_path, ssl_port);
                token_message = is_authed.unwrap();

                println!("Auth Token: {:?}", token_message.access_token);

                let twitch_tcp_conn_info = auth::TwitchTcpConnection {
                    access_token: token_message.access_token,
                    host: twitch_ssl_host_addrs,
                    port: ssl_port,
                };

                println!("About to call twitch_tcp_conn....");
                auth::Auth::twitch_tcp_conn(twitch_tcp_conn_info);
            }
            // let mut stream_buf = String::new();
            // let mut tcp_stream = stream.unwrap();
            // tcp_stream.read_to_string(&mut stream_buf).expect("Error reading stream");
            // println!("{}", stream_buf);

            // println!("Twitch TCP Conn: {}", twitch_tcp_conn);
            // });
        }

        // println!("Lets listen.....");
        // messages::Message::listen();
    }
}
