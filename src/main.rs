use elgato_keylight::KeyLight;
use std::error::Error;
use std::net::Ipv4Addr;
use std::str::FromStr;
use structopt::StructOpt;

const DEFAULT_IP_ADDRESS: &str = "192.168.0.16";

#[derive(StructOpt, Debug)]
#[structopt(name = "keylight")]
enum KeyLightCli {
    #[structopt(about = "Turns the keylight on with specified brightness and temperature")]
    On {
        #[structopt(short = "b", long = "brightness", default_value = "10")]
        brightness: u8,

        #[structopt(short = "t", long = "temperature", default_value = "3000")]
        temperature: u32,

        #[structopt(short = "i", long = "ip-address", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: String,
    },
    #[structopt(about = "Turns the keylight off")]
    Off {
        #[structopt(short = "i", long = "ip-address", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: String,
    },
    #[structopt(about = "Changes the brightness of the keylight by percentage (-100 to 100)")]
    Brightness {
        #[structopt(short = "b", long = "brightness")]
        brightness: i8,

        #[structopt(short = "i", long = "ip-address", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: String,
    },
    #[structopt(about = "Sets the temperature of the keylight")]
    Temperature {
        #[structopt(short = "t", long = "temperature")]
        temperature: u32,

        #[structopt(short = "i", long = "ip-address", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: String,
    },
}

async fn create_keylight(ip_address: String) -> Result<KeyLight, Box<dyn Error>> {
    let ip = Ipv4Addr::from_str(&ip_address)
        .map_err(|e| format!("Failed to parse IP address: {}", e))?;
    let kl = KeyLight::new_from_ip("Ring Light", ip, None)
        .await
        .map_err(|e| format!("Failed to create KeyLight: {}", e))?;
    Ok(kl)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = KeyLightCli::from_args();

    match args {
        KeyLightCli::On {
            brightness,
            temperature,
            ip_address,
        } => {
            let mut kl = create_keylight(ip_address).await?;

            kl.set_power(true).await?;
            kl.set_brightness(brightness).await?;
            kl.set_temperature(temperature).await?;
        }
        KeyLightCli::Off { ip_address } => {
            let mut kl = create_keylight(ip_address).await?;

            kl.set_power(false).await?;
        }
        KeyLightCli::Brightness {
            brightness,
            ip_address,
        } => {
            let mut kl = create_keylight(ip_address).await?;
            let relative_brightness = brightness as f64 / 100.0;

            kl.set_relative_brightness(relative_brightness).await?;
        }
        KeyLightCli::Temperature {
            temperature,
            ip_address,
        } => {
            let mut kl = create_keylight(ip_address).await?;

            kl.set_temperature(temperature).await?;
        }
    }

    Ok(())
}
