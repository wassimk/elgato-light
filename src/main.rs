use elgato_keylight::KeyLight;
use std::error::Error;
use std::net::Ipv4Addr;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "keylight")]
enum KeyLightCli {
    On,
    Off,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = KeyLightCli::from_args();

    let ip = Ipv4Addr::from_str("192.168.0.16")?;
    let mut kl = KeyLight::new_from_ip("Ring Light", ip, None).await?;

    match args {
        KeyLightCli::On => {
            kl.set_power(true).await?;
            kl.set_brightness(10).await?;
            kl.set_temperature(3000).await?;
        }
        KeyLightCli::Off => {
            kl.set_power(false).await?;
        }
    }

    let status = kl.get().await?;
    println!("{:?}", status);
    Ok(())
}
