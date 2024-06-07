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

async fn get_keylight(ip_address: String) -> Result<KeyLight, Box<dyn Error>> {
    let ip_address = Ipv4Addr::from_str(&ip_address)?;
    let keylight = KeyLight::new_from_ip("Ring Light", ip_address, None).await?;
    Ok(keylight)
}

fn get_ip_address(args: &KeyLightCli) -> String {
    match args {
        KeyLightCli::On { ip_address, .. } => ip_address.clone(),
        KeyLightCli::Off { ip_address } => ip_address.clone(),
        KeyLightCli::Brightness { ip_address, .. } => ip_address.clone(),
        KeyLightCli::Temperature { ip_address, .. } => ip_address.clone(),
        KeyLightCli::Status { ip_address } => ip_address.clone(),
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = KeyLightCli::from_args();
    let ip_address = get_ip_address(&args);

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
