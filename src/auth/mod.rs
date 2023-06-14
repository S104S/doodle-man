
pub struct Auth {
   pub key: String
}

impl Auth {
    pub fn twitch() -> String {
        println!("Authenticating with Twitch....");
        "Success".to_string()
    }
}