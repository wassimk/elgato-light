use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

const DEFAULT_IP_ADDRESS: &str = "192.168.0.25";
const LIGHT_PORT: u16 = 9123;

// --- Elgato Light HTTP API ---

#[derive(Debug, Serialize, Deserialize)]
struct LightStatus {
    #[serde(rename = "numberOfLights")]
    number_of_lights: i64,
    lights: Vec<Light>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Light {
    on: u8,
    brightness: u8,
    temperature: u16,
}

fn kelvin_to_mireds(kelvin: u32) -> u16 {
    (1_000_000 / kelvin) as u16
}

fn mireds_to_kelvin(mireds: u16) -> u32 {
    1_000_000 / mireds as u32
}

fn build_client() -> Result<Client> {
    Client::builder()
        // Force IPv4 — Elgato lights are IPv4-only local network devices.
        // reqwest 0.12 (hyper 1.0) can attempt IPv6 connections even for
        // IPv4 addresses, which fails with EHOSTUNREACH on some networks.
        .local_address(IpAddr::V4(Ipv4Addr::UNSPECIFIED))
        .http1_only()
        .build()
        .context("Failed to create HTTP client")
}

async fn get_status(client: &Client, ip: Ipv4Addr) -> Result<LightStatus> {
    let url = format!("http://{}:{}/elgato/lights", ip, LIGHT_PORT);
    let status: LightStatus = client
        .get(&url)
        .send()
        .await
        .context("Failed to connect to light")?
        .json()
        .await
        .context("Failed to parse light status")?;
    Ok(status)
}

async fn set_status(client: &Client, ip: Ipv4Addr, status: &LightStatus) -> Result<()> {
    let url = format!("http://{}:{}/elgato/lights", ip, LIGHT_PORT);
    client
        .put(&url)
        .json(status)
        .send()
        .await
        .context("Failed to send command to light")?;
    Ok(())
}

// --- CLI ---

fn validate_temperature(s: &str) -> Result<u32, String> {
    let val: u32 = s
        .parse()
        .map_err(|_| format!("'{s}' is not a valid number"))?;
    if (2900..=7000).contains(&val) {
        Ok(val)
    } else {
        Err(format!(
            "temperature must be between 2900 and 7000, got {val}"
        ))
    }
}

#[derive(Parser, Debug)]
#[command(
    name = "elgato-light",
    about = "A CLI for controlling an Elgato light by IP address",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Turn the light on with specified brightness and temperature
    On {
        #[arg(short, long, default_value = "10", value_parser = clap::value_parser!(u8).range(0..=100), help = "Brightness level (0-100)")]
        brightness: u8,

        #[arg(short, long, default_value = "3000", value_parser = validate_temperature, help = "Color temperature (2900-7000)")]
        temperature: u32,

        #[arg(short = 'i', long, env = "ELGATO_LIGHT_IP", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: Ipv4Addr,
    },
    /// Turn the light off
    Off {
        #[arg(short = 'i', long, env = "ELGATO_LIGHT_IP", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: Ipv4Addr,
    },
    /// Change the brightness relatively. Use -- to pass negative values.
    Brightness {
        #[arg(help = "Brightness change (-100 to 100)", allow_hyphen_values = true)]
        brightness: i8,

        #[arg(short = 'i', long, env = "ELGATO_LIGHT_IP", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: Ipv4Addr,
    },
    /// Set the color temperature
    Temperature {
        #[arg(value_parser = validate_temperature, help = "Color temperature (2900-7000)")]
        temperature: u32,

        #[arg(short = 'i', long, env = "ELGATO_LIGHT_IP", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: Ipv4Addr,
    },
    /// Get the current status of the light
    Status {
        #[arg(short = 'i', long, env = "ELGATO_LIGHT_IP", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: Ipv4Addr,
    },
}

impl Command {
    fn ip_address(&self) -> Ipv4Addr {
        match self {
            Command::On { ip_address, .. }
            | Command::Off { ip_address }
            | Command::Brightness { ip_address, .. }
            | Command::Temperature { ip_address, .. }
            | Command::Status { ip_address } => *ip_address,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let ip = cli.command.ip_address();
    let client = build_client()?;

    match cli.command {
        Command::On {
            brightness,
            temperature,
            ..
        } => {
            let status = LightStatus {
                number_of_lights: 1,
                lights: vec![Light {
                    on: 1,
                    brightness,
                    temperature: kelvin_to_mireds(temperature),
                }],
            };
            set_status(&client, ip, &status).await?;
            println!("Light on (brightness: {}%, temperature: {}K)", brightness, temperature);
        }
        Command::Off { .. } => {
            let status = LightStatus {
                number_of_lights: 1,
                lights: vec![Light {
                    on: 0,
                    brightness: 0,
                    temperature: 0,
                }],
            };
            set_status(&client, ip, &status).await?;
            println!("Light off");
        }
        Command::Brightness { brightness, .. } => {
            let mut status = get_status(&client, ip).await?;
            let light = status
                .lights
                .first_mut()
                .ok_or_else(|| anyhow!("No lights found in response"))?;
            if light.on == 0 {
                light.on = 1;
            }
            let new_brightness = (light.brightness as i16 + brightness as i16).clamp(0, 100) as u8;
            light.brightness = new_brightness;
            set_status(&client, ip, &status).await?;
            println!("Brightness: {}%", new_brightness);
        }
        Command::Temperature { temperature, .. } => {
            let mut status = get_status(&client, ip).await?;
            let light = status
                .lights
                .first_mut()
                .ok_or_else(|| anyhow!("No lights found in response"))?;
            if light.on == 0 {
                light.on = 1;
            }
            light.temperature = kelvin_to_mireds(temperature);
            set_status(&client, ip, &status).await?;
            println!("Temperature: {}K", temperature);
        }
        Command::Status { .. } => {
            let status = get_status(&client, ip).await?;
            let light = status
                .lights
                .first()
                .ok_or_else(|| anyhow!("No lights found in response"))?;
            println!(
                "Power:       {}",
                if light.on == 1 { "On" } else { "Off" }
            );
            println!("Brightness:  {}%", light.brightness);
            println!("Temperature: {}K", mireds_to_kelvin(light.temperature));
        }
    }

    Ok(())
}
