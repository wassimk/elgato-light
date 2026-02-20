use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;

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

fn light_url(ip: Ipv4Addr) -> String {
    format!("http://{}:{}/elgato/lights", ip, LIGHT_PORT)
}

fn get_status(ip: Ipv4Addr) -> Result<LightStatus> {
    let status: LightStatus = ureq::get(&light_url(ip))
        .call()
        .context("Failed to connect to light")?
        .into_json()
        .context("Failed to parse light status")?;
    Ok(status)
}

fn set_status(ip: Ipv4Addr, status: &LightStatus) -> Result<()> {
    ureq::put(&light_url(ip))
        .send_json(status)
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
    /// Turn the light on (use -b and -t to set brightness and temperature)
    On {
        #[arg(short, long, default_value = "10", value_parser = clap::value_parser!(u8).range(0..=100), help = "Brightness level (0-100)")]
        brightness: u8,

        #[arg(short, long, default_value = "5000", value_parser = validate_temperature, help = "Color temperature (2900-7000)")]
        temperature: u32,

        #[arg(short = 'i', long, env = "ELGATO_LIGHT_IP", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: Ipv4Addr,
    },
    /// Turn the light off
    Off {
        #[arg(short = 'i', long, env = "ELGATO_LIGHT_IP", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: Ipv4Addr,
    },
    /// Adjust brightness by a relative amount, e.g. 10 or -10
    Brightness {
        #[arg(help = "Relative adjustment (-100 to 100)", allow_hyphen_values = true)]
        brightness: i8,

        #[arg(short = 'i', long, env = "ELGATO_LIGHT_IP", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: Ipv4Addr,
    },
    /// Set the color temperature in Kelvin (2900-7000)
    Temperature {
        #[arg(value_parser = validate_temperature, help = "Temperature in Kelvin (2900-7000)")]
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

fn main() -> Result<()> {
    let cli = Cli::parse();
    let ip = cli.command.ip_address();

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
            set_status(ip, &status)?;
            println!(
                "Light on (brightness: {}%, temperature: {}K)",
                brightness, temperature
            );
        }
        Command::Off { .. } => {
            let mut status = get_status(ip)?;
            let light = status
                .lights
                .first_mut()
                .ok_or_else(|| anyhow!("No lights found in response"))?;
            light.on = 0;
            set_status(ip, &status)?;
            println!("Light off");
        }
        Command::Brightness { brightness, .. } => {
            let mut status = get_status(ip)?;
            let light = status
                .lights
                .first_mut()
                .ok_or_else(|| anyhow!("No lights found in response"))?;
            if light.on == 0 {
                light.on = 1;
            }
            let new_brightness = (light.brightness as i16 + brightness as i16).clamp(0, 100) as u8;
            light.brightness = new_brightness;
            set_status(ip, &status)?;
            println!("Brightness: {}%", new_brightness);
        }
        Command::Temperature { temperature, .. } => {
            let mut status = get_status(ip)?;
            let light = status
                .lights
                .first_mut()
                .ok_or_else(|| anyhow!("No lights found in response"))?;
            if light.on == 0 {
                light.on = 1;
            }
            light.temperature = kelvin_to_mireds(temperature);
            set_status(ip, &status)?;
            println!("Temperature: {}K", temperature);
        }
        Command::Status { .. } => {
            let status = get_status(ip)?;
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
