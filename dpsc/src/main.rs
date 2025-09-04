use clap::{Subcommand, Parser};
use std::fs::{self, OpenOptions};
use std::io::{Write};
use serde::{Deserialize, Serialize};

#[derive(Parser)]
#[command(version, about)]
struct CLI {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Config {
        #[arg(short, long)]
        addr: Option<String>,
        #[arg(short, long)]
        featureflags: Option<u8>
    },
    Status {

    } ,
    List {

    }
}

// CONFIG START
#[derive(Deserialize, Serialize, Debug)]
struct Configuration {
    server: ServerConfiguration
}
#[derive(Deserialize, Serialize, Debug)]
struct ServerConfiguration {
    address: String,
    featureflags: u8,
}

// CONFIG END

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            server: ServerConfiguration { address: "127.0.0.1:8487".to_string(), featureflags: 0b0000_0001 },
        }
    }
}

fn main() {
    let cli = CLI::parse();

    generate_empty();

    let mut configuration = check_config().unwrap();

    match &cli.command {
        Some(Commands::Config { addr , featureflags}) => {
            //This match change updates their configurations if they exist. Sorry for ugly :(

            match addr {
                Some(address) => {
                    configuration.server.address = address.clone();
                }
                None => {}
            }
            match featureflags {
                Some(flags) => {
                    configuration.server.featureflags = flags.clone();
                }
                None => {}
            }

            save_config(&configuration).unwrap();
        },
        Some(Commands::Status {  }) => {
            println!("Sus");
        },
        Some(Commands::List {  }) => {
            println!("List");
        },
        None => {
            println!("hi!");
        }
    }


}
fn check_config() -> Result<Configuration, Box<dyn std::error::Error>> {
    fs::create_dir_all("/etc/dpp")?;
    let mut file = OpenOptions::new().read(true).write(true).create(true).open("/etc/dpp/config.toml")?;

    let content = fs::read_to_string("/etc/dpp/config.toml")?;

    let configuration: Configuration = if content.trim().is_empty() {
        let default = Configuration::default();
        let toml_str = toml::to_string_pretty(&default).unwrap();

        file.write_all(toml_str.as_bytes())?;
        println!("No configuration found, so one was created.");
        default

    } else {
        match toml::from_str(&content) {
            Ok(configuration) => configuration,
            Err(_) => {
                let default = Configuration::default();
                let toml_str = toml::to_string_pretty(&default).unwrap();

                file.write_all(toml_str.as_bytes())?;
                println!("Invalid configuration found, so one was created.");
                default
            }
        }
    };

    Ok(configuration)
}

fn save_config(configuration: &Configuration) -> Result<(), Box<dyn std::error::Error>> {
    let toml_str = toml::to_string_pretty(configuration)?;

    fs::write("/etc/dpp/.config.toml.tmp", toml_str.as_bytes())?;
    fs::rename("/etc/dpp/.config.toml.tmp", "/etc/dpp/config.toml")?;

    Ok(())
}

fn generate_empty() {
    let _server_list = OpenOptions::new().create(true).open("/etc/dpp/server_list.toml");
}