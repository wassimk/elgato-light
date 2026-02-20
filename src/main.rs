use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::net::Ipv4Addr;
#[cfg(target_os = "macos")]
use std::path::PathBuf;

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

struct LightTarget {
    ip: Ipv4Addr,
    port: u16,
}

fn kelvin_to_mireds(kelvin: u32) -> u16 {
    (1_000_000 / kelvin) as u16
}

fn mireds_to_kelvin(mireds: u16) -> u32 {
    1_000_000 / mireds as u32
}

fn light_url(target: &LightTarget) -> String {
    format!("http://{}:{}/elgato/lights", target.ip, target.port)
}

fn get_status(target: &LightTarget) -> Result<LightStatus> {
    let status: LightStatus = ureq::get(&light_url(target))
        .call()
        .context("Failed to connect to light")?
        .into_json()
        .context("Failed to parse light status")?;
    Ok(status)
}

fn set_status(target: &LightTarget, status: &LightStatus) -> Result<()> {
    ureq::put(&light_url(target))
        .send_json(status)
        .context("Failed to send command to light")?;
    Ok(())
}

// --- Discovery ---

fn resolve_target(ip: Option<Ipv4Addr>) -> Result<LightTarget> {
    match ip {
        Some(ip) => Ok(LightTarget {
            ip,
            port: LIGHT_PORT,
        }),
        None => resolve_target_no_ip(),
    }
}

#[cfg(target_os = "macos")]
fn resolve_target_no_ip() -> Result<LightTarget> {
    if let Some(cached) = read_cached_target() {
        if get_status(&cached).is_ok() {
            return Ok(cached);
        }
        clear_cached_target();
        eprintln!("Cached light unreachable, rediscovering...");
    }

    let target = discover_light()?;
    write_cached_target(&target);
    Ok(target)
}

#[cfg(not(target_os = "macos"))]
fn resolve_target_no_ip() -> Result<LightTarget> {
    Err(anyhow!(
        "No IP address specified.\n\
         Use --ip-address <IP> or set the ELGATO_LIGHT_IP environment variable."
    ))
}

// --- Cache ---

#[cfg(target_os = "macos")]
fn cache_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join("Library/Caches/elgato-light/target"))
}

#[cfg(target_os = "macos")]
fn read_cached_target() -> Option<LightTarget> {
    let content = std::fs::read_to_string(cache_path()?).ok()?;
    let mut parts = content.trim().splitn(2, ':');
    let ip = parts.next()?.parse().ok()?;
    let port = parts.next()?.parse().ok()?;
    Some(LightTarget { ip, port })
}

#[cfg(target_os = "macos")]
fn write_cached_target(target: &LightTarget) {
    if let Some(path) = cache_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, format!("{}:{}", target.ip, target.port));
    }
}

#[cfg(target_os = "macos")]
fn clear_cached_target() {
    if let Some(path) = cache_path() {
        let _ = std::fs::remove_file(path);
    }
}

#[cfg(target_os = "macos")]
fn discover_light() -> Result<LightTarget> {
    use mdns_sd::{ServiceDaemon, ServiceEvent};
    use std::time::{Duration, Instant};

    const SERVICE_TYPE: &str = "_elg._tcp.local.";
    const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(5);

    eprintln!("Discovering Elgato lights on the network...");

    let mdns = ServiceDaemon::new().context("Failed to start mDNS daemon")?;
    let receiver = mdns
        .browse(SERVICE_TYPE)
        .context("Failed to browse for Elgato lights")?;
    let deadline = Instant::now() + DISCOVERY_TIMEOUT;

    while let Ok(event) = receiver.recv_deadline(deadline) {
        if let ServiceEvent::ServiceResolved(info) = event {
            let valid_ip = info
                .get_addresses_v4()
                .into_iter()
                .find(|v4| !v4.is_loopback() && !v4.is_unspecified());

            if let Some(ip) = valid_ip {
                let target = LightTarget {
                    ip,
                    port: info.get_port(),
                };
                eprintln!(
                    "Found: {} ({}:{})",
                    info.get_fullname(),
                    target.ip,
                    target.port
                );
                let _ = mdns.stop_browse(SERVICE_TYPE);
                let _ = mdns.shutdown();
                return Ok(target);
            }
        }
    }

    let _ = mdns.stop_browse(SERVICE_TYPE);
    let _ = mdns.shutdown();

    Err(anyhow!(
        "No Elgato light found on the network within {}s.\n\
         Make sure the light is powered on and connected to the same network.\n\
         Alternatively, specify the IP with: --ip-address <IP>",
        DISCOVERY_TIMEOUT.as_secs()
    ))
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
    about = "A CLI for controlling an Elgato light (auto-discovers via Bonjour on macOS)",
    version
)]
struct Cli {
    #[arg(short = 'i', long, global = true, env = "ELGATO_LIGHT_IP", help = "IP address of the light (auto-discovered on macOS if omitted)")]
    ip_address: Option<Ipv4Addr>,

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
    },
    /// Turn the light off
    Off,
    /// Adjust brightness by a relative amount, e.g. 10 or -10
    Brightness {
        #[arg(help = "Relative adjustment (-100 to 100)", allow_hyphen_values = true)]
        brightness: i8,
    },
    /// Set the color temperature in Kelvin (2900-7000)
    Temperature {
        #[arg(value_parser = validate_temperature, help = "Temperature in Kelvin (2900-7000)")]
        temperature: u32,
    },
    /// Get the current status of the light
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let target = resolve_target(cli.ip_address)?;

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
            set_status(&target, &status)?;
            println!(
                "Light on (brightness: {}%, temperature: {}K)",
                brightness, temperature
            );
        }
        Command::Off => {
            let mut status = get_status(&target)?;
            let light = status
                .lights
                .first_mut()
                .ok_or_else(|| anyhow!("No lights found in response"))?;
            light.on = 0;
            set_status(&target, &status)?;
            println!("Light off");
        }
        Command::Brightness { brightness, .. } => {
            let mut status = get_status(&target)?;
            let light = status
                .lights
                .first_mut()
                .ok_or_else(|| anyhow!("No lights found in response"))?;
            if light.on == 0 {
                light.on = 1;
            }
            let new_brightness = (light.brightness as i16 + brightness as i16).clamp(0, 100) as u8;
            light.brightness = new_brightness;
            set_status(&target, &status)?;
            println!("Brightness: {}%", new_brightness);
        }
        Command::Temperature { temperature, .. } => {
            let mut status = get_status(&target)?;
            let light = status
                .lights
                .first_mut()
                .ok_or_else(|| anyhow!("No lights found in response"))?;
            if light.on == 0 {
                light.on = 1;
            }
            light.temperature = kelvin_to_mireds(temperature);
            set_status(&target, &status)?;
            println!("Temperature: {}K", temperature);
        }
        Command::Status => {
            let status = get_status(&target)?;
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
