# keylight-cli

CLI tool to control an Elgato light

The Elgato Control Central software rarely detects my Elgato Ring Light, and I'm at my wits' end trying to find out why. I've tried everything, even setting up a separate 2.4 GHz wireless network for it.

During troubleshooting, I learned it always has an IP address and never loses a network connection. That's what led me to try to find its interface. So, here we are. I'm going to control it with this CLI tool via Raycast on macOS.

### Usage

For now, it's a simple CLI tool with my IP address, brightness, and temperature hard-coded. I will improve it as needed.

```shell
keylight on
keylight off
```
