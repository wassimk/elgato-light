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
    #[structopt(
        about = "Changes the brightness of the keylight. Values are -100 to 100. Use -- to pass negative arguments."
    )]
    Brightness {
        #[structopt()]
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
    #[structopt(about = "Gets the status of the keylight")]
    Status {
        #[structopt(short = "i", long = "ip-address", default_value = DEFAULT_IP_ADDRESS)]
        ip_address: String,
    },
}

impl KeyLightCli {
    fn ip_address(&self) -> Result<Ipv4Addr, Box<dyn Error>> {
        let ip_str = match self {
            KeyLightCli::On { ip_address, .. } => ip_address,
            KeyLightCli::Off { ip_address } => ip_address,
            KeyLightCli::Brightness { ip_address, .. } => ip_address,
            KeyLightCli::Temperature { ip_address, .. } => ip_address,
            KeyLightCli::Status { ip_address } => ip_address,
        };

        Ipv4Addr::from_str(ip_str).map_err(|_| "Invalid IP address format".into())
    }
}

async fn get_keylight(ip_address: Ipv4Addr) -> Result<KeyLight, Box<dyn Error>> {
    let keylight = KeyLight::new_from_ip("Elgato Light", ip_address, None).await?;
    Ok(keylight)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = KeyLightCli::from_args();
    let ip_address = args.ip_address()?;
    let mut keylight = get_keylight(ip_address).await?;

    match args {
        KeyLightCli::On {
            brightness,
            temperature,
            ..
        } => {
            keylight.set_power(true).await?;
            keylight.set_brightness(brightness).await?;
            keylight.set_temperature(temperature).await?;
        }
        KeyLightCli::Off { .. } => {
            keylight.set_power(false).await?;
        }
        KeyLightCli::Brightness { brightness, .. } => {
            let status = keylight.get().await?;
            if status.lights[0].on == 0 {
                keylight.set_power(true).await?;
            }

            let current_brightness = status.lights[0].brightness;
            let new_brightness = ((current_brightness as i8) + brightness).clamp(0, 100) as u8;
            keylight.set_brightness(new_brightness).await?;
        }
        KeyLightCli::Temperature { temperature, .. } => {
            let status = keylight.get().await?;
            if status.lights[0].on == 0 {
                keylight.set_power(true).await?;
            }

            keylight.set_temperature(temperature).await?;
        }
        KeyLightCli::Status { .. } => {
            let status = keylight.get().await?;
            println!("{:?}", status);
        }
    }

    Ok(())
}
