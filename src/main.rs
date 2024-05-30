use elgato_keylight::KeyLight;
use std::error::Error;
use std::net::Ipv4Addr;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "keylight")]
enum KeyLightCli {
    On {
        #[structopt(short = "b", long = "brightness", default_value = "10")]
        brightness: u8,

        #[structopt(short = "t", long = "temperature", default_value = "3000")]
        temperature: u32,

        #[structopt(short = "i", long = "ip-address", default_value = "192.168.0.16")]
        ip_address: String,
    },
    Off {
        #[structopt(short = "i", long = "ip-address", default_value = "192.168.0.16")]
        ip_address: String,
    },
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
            let ip = Ipv4Addr::from_str(&ip_address)?;
            let mut kl = KeyLight::new_from_ip("Ring Light", ip, None).await?;

            kl.set_power(true).await?;
            kl.set_brightness(brightness).await?;
            kl.set_temperature(temperature).await?;
        }
        KeyLightCli::Off { ip_address } => {
            let ip = Ipv4Addr::from_str(&ip_address)?;
            let mut kl = KeyLight::new_from_ip("Ring Light", ip, None).await?;

            kl.set_power(false).await?;
        }
    }

    Ok(())
}
