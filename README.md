# elgato-light

A CLI tool for controlling Elgato lights.

On macOS, lights are automatically discovered via Bonjour with support for multiple lights. On Linux, an IP address must be provided. This tool can be used stand-alone, but it was designed primarily for use within an extension with [Raycast](https://www.raycast.com) on macOS. That extension is available in the [raycast-elgato-light](https://github.com/wassimk/raycast-elgato-light) repository.

### Features

- **Multi-light support** with automatic discovery of all lights on the network
- **Auto-discovery** on macOS via Bonjour/mDNS, no configuration needed
- **Discovery caching** for instant subsequent runs
- **Manual IP** support via `--ip-address` flag or `ELGATO_LIGHT_IP` environment variable, comma-separated for multiple lights
- **Power control** with `on` and `off` commands
- **Brightness** control, absolute on turn-on or relative adjustments
- **Color temperature** control in Kelvin (2900K to 7000K)
- **Status** reporting of current light state with light name

### Usage

```
$ elgato-light --help

A CLI for controlling Elgato lights (auto-discovers via Bonjour on macOS)

Usage: elgato-light [OPTIONS] <COMMAND>

Commands:
  on           Turn the light(s) on (use -b and -t to set brightness and temperature)
  off          Turn the light(s) off
  brightness   Adjust brightness by a relative amount, e.g. 10 or -10
  temperature  Set the color temperature in Kelvin (2900-7000)
  status       Get the current status of the light(s)
  discover     Discover lights on the network and save for future use
  clear-cache  Clear the saved lights cache

Options:
  -i, --ip-address <IP_ADDRESS>  Light IP address(es), comma-separated
  -l, --light <LIGHT>            Target a specific light by name (case-insensitive substring match)
      --timeout <TIMEOUT>        Discovery timeout in seconds [default: 10]
  -h, --help                     Print help
  -V, --version                  Print version
```

On macOS, no configuration is needed. The CLI discovers all Elgato lights automatically on first run and saves them for instant subsequent runs.

```shell
elgato-light on
elgato-light off
```

Brightness and/or temperature can be set when turning on. Brightness defaults to 10% and temperature to 5000K.

```shell
elgato-light on --brightness 20
elgato-light on --temperature 5000
elgato-light on --brightness 20 --temperature 5000
```

Adjust relative brightness between 0 and 100. *Use `--` before negative values.*

```shell
elgato-light brightness -- 10
elgato-light brightness -- -10
```

Set the color temperature in Kelvin (2900 to 7000).

```shell
elgato-light temperature 5000
```

Get the current light status.

```shell
elgato-light status
```

### Multiple Lights

When multiple lights are discovered, all commands target every light by default.

Discover and save lights on the network.

```shell
elgato-light discover
```

Target a specific light by name (case-insensitive substring match).

```shell
elgato-light --light "Key Light" on
elgato-light -l AB12 status
```

### Specifying an IP Address

On Linux, or to skip auto-discovery on macOS, provide the light's IP address directly. Supports comma-separated values for multiple lights.

```shell
elgato-light --ip-address 192.168.0.10 on
elgato-light --ip-address 192.168.0.10,192.168.0.11 on
```

The `ELGATO_LIGHT_IP` environment variable can be set as an alternative to passing `--ip-address` on every command.

```shell
export ELGATO_LIGHT_IP=192.168.0.10,192.168.0.11
elgato-light on
```

### Troubleshooting

If auto-discovery is not finding your light, verify Bonjour can see it with the macOS built-in tool.

```shell
dns-sd -B _elg._tcp
```

Re-run discovery if lights have changed on the network.

```shell
elgato-light discover
```

Clear the saved lights cache.

```shell
elgato-light clear-cache
```

The discovery cache is stored at `~/Library/Caches/elgato-light/target`.

The Apple binaries are signed and notarized with an Apple Developer account, so they should work without any Gatekeeper warnings.
