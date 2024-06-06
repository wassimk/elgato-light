# keylight-cli

This is a CLI tool to control an Elgato light.

The Elgato Control Central software rarely detects my Elgato Ring Light, and I'm at my wits' end trying to find out why. I've tried everything, including a separate 2.4 GHz wireless network.

During troubleshooting, I learned it always has an IP address and never loses a network connection. That's what led me to find its interface. So, here we are. I will control it with this CLI tool via Raycast on macOS.

### Usage

It's a simple CLI tool with my IP address, brightness, and temperature hard-coded as the defaults.

```shell
keylight on
keylight off
```

Specify the action settings via CLI arguments.

```shell
keylight on --ip-address 192.168.0.16 --brightness 10 --temperature 3000
keylight off --ip-address 192.168.0.16
```

Also, it supports increasing or decreasing the brightness by a percentage.
```shell
keylight brightness 10 --ip-address 192.168.0.16
keylight brightness -10 --ip-address 192.168.0.16
```
