# elgato-light

This is a CLI tool to control an Elgato light. It also works for any of their key lights.

The Elgato Control Central software rarely detects my Elgato Ring Light, and I'm at my wits' end trying to find out why. I've tried everything, including a separate 2.4 GHz wireless network.

During troubleshooting, I learned it always has an IP address and never loses a network connection. That's what led me to find its interface. So, here we are. I will control it with this CLI tool via Raycast on macOS.

### Usage

It's a CLI tool with my Elgato light IP address of *192.168.0.25*, brightness, and temperature hard-coded as the defaults.

```shell
elgato-light on
elgato-light off
```

Brightness and/or temperature can be set when turning on.

```shell
elgato-light on --brightness 20
elgato-light on --temperature 5000
elgato-light on --brightness 20 --temperature 5000
```

Change the relative brightness between 0 and 100. *Use `--` for negative values.*

```shell
elgato-light brightness -- 10
elgato-light brightness -- -10
```

Set the temperature between 2900 and 7000.

```shell
elgato-light temperature 5000
```

Use a non-default IP address for the light on all commands.

```shell
elgato-light on --ip-address 192.168.0.10
elgato-light off --ip-address 192.168.0.10
```

Help is available for all commands.

```shell
elgato-light --help
elgato-light on --help
elgato-light brightness --help
```

### Troubleshooting

Get the light status.

```shell
elgato-light status
```

The Apple binaries are not signed with an Apple Developer account, so you must authorize them manually.

```shell
xattr -dr com.apple.quarantine ./elgato-light
```
