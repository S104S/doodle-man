use std::thread;

mod auth;

fn main() {

    let channel = std::env::args().nth(1).expect("no channel entered");

    thread::spawn(|| {
       auth::Auth::twitch();
    });

    println!("{}", channel.to_string());
    println!("Hello, world!");
}
