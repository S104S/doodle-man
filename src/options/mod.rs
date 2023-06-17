use std::env;
use std::env::Args;
use std::path::{Path, PathBuf};

pub struct Options {
    pub channel: String
}

impl Options {

    fn get(arg: &str) -> Result<bool, String> {
        match arg {
            "channel" => Ok(true),
            _ => Err(format!("invalid CLI argument"))
        }
    }

    pub fn parse_args (args: Args) -> Self {
        let parsible_args: Vec<String> = args.collect();
        let mut settings_prop: String;

        for mut parsible_arg in parsible_args {
            if (parsible_arg.starts_with("--") || parsible_arg.starts_with("-")) && parsible_arg.len() >= 3 {
                let mut final_arg = &parsible_arg[2..];;
                let mut offset = parsible_arg.find("--").unwrap_or(0);

                if offset == 0 {
                    final_arg = &parsible_arg[1..];
                }
                println!("Arg: {}",final_arg.to_string());


                let argument_found = Options::get( &final_arg).unwrap();
                println!("{}", argument_found);
            }
        }

        let final_settings = Options{
            channel: "".to_string()
        };

        final_settings
    }
}