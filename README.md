# elgato-light

A CLI tool for controlling Elgato lights.

On macOS, lights are automatically discovered via Bonjour. On Linux, an IP address must be provided. This tool can be used stand-alone, but it was designed primarily for use within an extension with [Raycast](https://www.raycast.com) on macOS. That extension is available in the [raycast-elgato-light](https://github.com/wassimk/raycast-elgato-light) repository.

### 💡 Features

- **Auto-discovery** on macOS via Bonjour/mDNS, no configuration needed
- **Discovery caching** for instant subsequent runs, automatically cleared when the light becomes unreachable
- **Manual IP** support via `--ip-address` flag or `ELGATO_LIGHT_IP` environment variable
- **Power control** with `on` and `off` commands
- **Brightness** control, absolute on turn-on or relative adjustments
- **Color temperature** control in Kelvin (2900K to 7000K)
- **Status** reporting of current light state

### 💻 Usage

```
$ elgato-light --help

A CLI for controlling an Elgato light (auto-discovers via Bonjour on macOS)

Usage: elgato-light [OPTIONS] <COMMAND>

Commands:
  on           Turn the light on (use -b and -t to set brightness and temperature)
  off          Turn the light off
  brightness   Adjust brightness by a relative amount, e.g. 10 or -10
  temperature  Set the color temperature in Kelvin (2900-7000)
  status       Get the current status of the light

Options:
  -i, --ip-address <IP_ADDRESS>  IP address of the light (auto-discovered on macOS if omitted)
  -h, --help                     Print help
  -V, --version                  Print version
```

On macOS, no configuration is needed. The CLI discovers your Elgato light automatically and caches the result for instant subsequent runs.

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

### 🌐 Specifying an IP Address

On Linux, or to skip auto-discovery on macOS, provide the light's IP address directly.

```shell
elgato-light --ip-address 192.168.0.10 on
elgato-light --ip-address 192.168.0.10 status
```

The `ELGATO_LIGHT_IP` environment variable can be set as an alternative to passing `--ip-address` on every command.

```shell
export ELGATO_LIGHT_IP=192.168.0.10
elgato-light on
```

### 🔍 Troubleshooting

If auto-discovery is not finding your light, verify Bonjour can see it with the macOS built-in tool.

```shell
dns-sd -B _elg._tcp
```

The discovery cache is stored at `~/Library/Caches/elgato-light/target` and is automatically cleared when the cached light becomes unreachable. To manually clear it:

```shell
rm ~/Library/Caches/elgato-light/target
```

The Apple binaries are signed and notarized with an Apple Developer account, so they should work without any Gatekeeper warnings.
