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
    name: String,
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
        .with_context(|| format!("Failed to connect to light '{}'", target.name))?
        .into_json()
        .with_context(|| format!("Failed to parse status from light '{}'", target.name))?;
    Ok(status)
}

fn set_status(target: &LightTarget, status: &LightStatus) -> Result<()> {
    ureq::put(&light_url(target))
        .send_json(status)
        .with_context(|| format!("Failed to send command to light '{}'", target.name))?;
    Ok(())
}

// --- Discovery ---

fn parse_ips(s: &str) -> Result<Vec<Ipv4Addr>> {
    s.split(',')
        .map(|part| {
            part.trim()
                .parse::<Ipv4Addr>()
                .with_context(|| format!("'{}' is not a valid IPv4 address", part.trim()))
        })
        .collect()
}

fn resolve_targets(ip_str: Option<&str>, light_filter: Option<&str>, timeout_secs: u64) -> Result<Vec<LightTarget>> {
    if ip_str.is_some() && light_filter.is_some() {
        return Err(anyhow!("Cannot use --ip-address and --light together"));
    }

    match ip_str {
        Some(s) => {
            let ips = parse_ips(s)?;
            Ok(ips
                .into_iter()
                .map(|ip| LightTarget {
                    name: ip.to_string(),
                    ip,
                    port: LIGHT_PORT,
                })
                .collect())
        }
        None => {
            let targets = resolve_targets_no_ip(timeout_secs)?;
            match light_filter {
                Some(filter) => {
                    let filter_lower = filter.to_lowercase();
                    let filtered: Vec<_> = targets
                        .into_iter()
                        .filter(|t| t.name.to_lowercase().contains(&filter_lower))
                        .collect();
                    if filtered.is_empty() {
                        Err(anyhow!(
                            "No light found matching '{}'. Use 'discover' to see available lights.",
                            filter
                        ))
                    } else {
                        Ok(filtered)
                    }
                }
                None => Ok(targets),
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn resolve_targets_no_ip(timeout_secs: u64) -> Result<Vec<LightTarget>> {
    if let Some(cached) = read_cached_targets() {
        return Ok(cached);
    }

    eprintln!("No saved lights, discovering...");
    let targets = discover_lights(timeout_secs)?;
    write_cached_targets(&targets);
    Ok(targets)
}

#[cfg(not(target_os = "macos"))]
fn resolve_targets_no_ip(_timeout_secs: u64) -> Result<Vec<LightTarget>> {
    Err(anyhow!(
        "No IP address specified.\n\
         Use --ip-address <IP> or set the ELGATO_LIGHT_IP environment variable."
    ))
}

// --- Cache ---

#[cfg(target_os = "macos")]
#[derive(Debug, Serialize, Deserialize)]
struct CachedLight {
    name: String,
    ip: String,
    port: u16,
}

#[cfg(target_os = "macos")]
fn cache_path() -> Option<PathBuf> {
    std::env::var("HOME")
        .ok()
        .map(|home| PathBuf::from(home).join("Library/Caches/elgato-light/target"))
}

#[cfg(target_os = "macos")]
fn read_cached_targets() -> Option<Vec<LightTarget>> {
    let content = std::fs::read_to_string(cache_path()?).ok()?;
    let cached: Vec<CachedLight> = serde_json::from_str(&content).ok()?;
    if cached.is_empty() {
        return None;
    }
    let targets: Vec<LightTarget> = cached
        .into_iter()
        .filter_map(|c| {
            let ip = c.ip.parse().ok()?;
            Some(LightTarget {
                name: c.name,
                ip,
                port: c.port,
            })
        })
        .collect();
    if targets.is_empty() {
        None
    } else {
        Some(targets)
    }
}

#[cfg(target_os = "macos")]
fn write_cached_targets(targets: &[LightTarget]) {
    if let Some(path) = cache_path() {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let cached: Vec<CachedLight> = targets
            .iter()
            .map(|t| CachedLight {
                name: t.name.clone(),
                ip: t.ip.to_string(),
                port: t.port,
            })
            .collect();
        if let Ok(json) = serde_json::to_string_pretty(&cached) {
            let _ = std::fs::write(path, json);
        }
    }
}

#[cfg(target_os = "macos")]
fn clear_cached_targets() {
    if let Some(path) = cache_path() {
        let _ = std::fs::remove_file(path);
    }
}

// --- mDNS Discovery ---

#[cfg(target_os = "macos")]
fn extract_light_name(fullname: &str) -> String {
    fullname
        .strip_suffix("._elg._tcp.local.")
        .unwrap_or(fullname)
        .to_string()
}

#[cfg(target_os = "macos")]
fn discover_lights(timeout_secs: u64) -> Result<Vec<LightTarget>> {
    use mdns_sd::{ServiceDaemon, ServiceEvent};
    use std::time::{Duration, Instant};

    const SERVICE_TYPE: &str = "_elg._tcp.local.";

    let timeout = Duration::from_secs(timeout_secs);

    eprintln!("Discovering Elgato lights on the network...");

    let mdns = ServiceDaemon::new().context("Failed to start mDNS daemon")?;
    let receiver = mdns
        .browse(SERVICE_TYPE)
        .context("Failed to browse for Elgato lights")?;
    let deadline = Instant::now() + timeout;

    let mut targets = Vec::new();

    while let Ok(event) = receiver.recv_deadline(deadline) {
        if let ServiceEvent::ServiceResolved(info) = event {
            let valid_ip = info
                .get_addresses_v4()
                .into_iter()
                .find(|v4| !v4.is_loopback() && !v4.is_unspecified());

            if let Some(ip) = valid_ip {
                let name = extract_light_name(info.get_fullname());
                let target = LightTarget {
                    name,
                    ip,
                    port: info.get_port(),
                };
                eprintln!("  Found: {} ({}:{})", target.name, target.ip, target.port);
                targets.push(target);
            }
        }
    }

    let _ = mdns.stop_browse(SERVICE_TYPE);
    let _ = mdns.shutdown();

    if targets.is_empty() {
        Err(anyhow!(
            "No Elgato lights found on the network within {}s.\n\
             Make sure your lights are powered on and connected to the same network.\n\
             Alternatively, specify IP(s) with: --ip-address <IP>",
            timeout.as_secs()
        ))
    } else {
        targets.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(targets)
    }
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
    about = "A CLI for controlling Elgato lights (auto-discovers via Bonjour on macOS)",
    version
)]
struct Cli {
    #[arg(short = 'i', long, global = true, env = "ELGATO_LIGHT_IP",
           help = "Light IP address(es), comma-separated")]
    ip_address: Option<String>,

    #[arg(short = 'l', long, global = true,
           help = "Target a specific light by name (case-insensitive substring match)")]
    light: Option<String>,

    #[arg(long, global = true, default_value = "10",
           help = "Discovery timeout in seconds")]
    timeout: u64,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Turn the light(s) on (use -b and -t to set brightness and temperature)
    On {
        #[arg(short, long, default_value = "10", value_parser = clap::value_parser!(u8).range(0..=100), help = "Brightness level (0-100)")]
        brightness: u8,

        #[arg(short, long, default_value = "5000", value_parser = validate_temperature, help = "Color temperature (2900-7000)")]
        temperature: u32,
    },
    /// Turn the light(s) off
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
    /// Get the current status of the light(s)
    Status,
    /// Discover lights on the network and save for future use
    Discover,
    /// Clear the saved lights cache
    ClearCache,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if matches!(cli.command, Command::Discover) {
        return run_discover(cli.timeout);
    }

    if matches!(cli.command, Command::ClearCache) {
        return run_clear_cache();
    }

    let targets = resolve_targets(cli.ip_address.as_deref(), cli.light.as_deref(), cli.timeout)?;
    let multi = targets.len() > 1;

    match cli.command {
        Command::On {
            brightness,
            temperature,
        } => {
            for target in &targets {
                let status = LightStatus {
                    number_of_lights: 1,
                    lights: vec![Light {
                        on: 1,
                        brightness,
                        temperature: kelvin_to_mireds(temperature),
                    }],
                };
                set_status(target, &status)?;
            }
            println!(
                "Light on (brightness: {}%, temperature: {}K){}",
                brightness,
                temperature,
                if multi {
                    format!(" [{} lights]", targets.len())
                } else {
                    String::new()
                }
            );
        }
        Command::Off => {
            for target in &targets {
                let mut status = get_status(target)?;
                let light = status
                    .lights
                    .first_mut()
                    .ok_or_else(|| anyhow!("No lights in response from '{}'", target.name))?;
                light.on = 0;
                set_status(target, &status)?;
            }
            println!(
                "Light off{}",
                if multi {
                    format!(" [{} lights]", targets.len())
                } else {
                    String::new()
                }
            );
        }
        Command::Brightness { brightness } => {
            for target in &targets {
                let mut status = get_status(target)?;
                let light = status
                    .lights
                    .first_mut()
                    .ok_or_else(|| anyhow!("No lights in response from '{}'", target.name))?;
                if light.on == 0 {
                    light.on = 1;
                }
                let new_brightness =
                    (light.brightness as i16 + brightness as i16).clamp(0, 100) as u8;
                light.brightness = new_brightness;
                set_status(target, &status)?;
                if multi {
                    println!("{}: brightness {}%", target.name, new_brightness);
                } else {
                    println!("Brightness: {}%", new_brightness);
                }
            }
        }
        Command::Temperature { temperature } => {
            for target in &targets {
                let mut status = get_status(target)?;
                let light = status
                    .lights
                    .first_mut()
                    .ok_or_else(|| anyhow!("No lights in response from '{}'", target.name))?;
                if light.on == 0 {
                    light.on = 1;
                }
                light.temperature = kelvin_to_mireds(temperature);
                set_status(target, &status)?;
            }
            println!(
                "Temperature: {}K{}",
                temperature,
                if multi {
                    format!(" [{} lights]", targets.len())
                } else {
                    String::new()
                }
            );
        }
        Command::Status => {
            for (i, target) in targets.iter().enumerate() {
                if i > 0 {
                    println!();
                }
                let status = get_status(target)?;
                let light = status
                    .lights
                    .first()
                    .ok_or_else(|| anyhow!("No lights in response from '{}'", target.name))?;
                println!("Name:        {}", target.name);
                println!(
                    "Power:       {}",
                    if light.on == 1 { "On" } else { "Off" }
                );
                println!("Brightness:  {}%", light.brightness);
                println!("Temperature: {}K", mireds_to_kelvin(light.temperature));
            }
        }
        Command::Discover | Command::ClearCache => unreachable!(),
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn run_discover(timeout_secs: u64) -> Result<()> {
    clear_cached_targets();
    let targets = discover_lights(timeout_secs)?;
    write_cached_targets(&targets);
    println!("Found {} light(s):", targets.len());
    for target in &targets {
        println!("  {} ({}:{})", target.name, target.ip, target.port);
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn run_discover(_timeout_secs: u64) -> Result<()> {
    Err(anyhow!(
        "Discovery requires macOS with Bonjour.\n\
         Use --ip-address <IP> to specify light(s) directly."
    ))
}

#[cfg(target_os = "macos")]
fn run_clear_cache() -> Result<()> {
    clear_cached_targets();
    println!("Saved lights cache cleared.");
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn run_clear_cache() -> Result<()> {
    println!("No cache to clear (discovery is macOS only).");
    Ok(())
}
