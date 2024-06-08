# elgato-light

This is a CLI tool designed to control an Elgato light.

As someone who uses an Elgato Ring Light, I struggled with the Elgato Control Central software, which often fails to detect my light. After countless attempts to fix the issue, including setting up a 2.4 GHz network specifically for the light, I finally found a solution.

During my troubleshooting, I discovered that the light consistently holds an IP address, ensuring its continuous connection to the network. This revelation led me to delve deeper into the light's interface, ultimately resulting in the development of this CLI tool.

This tool can be used stand-alone, but I designed it primarily for use within an extension with [Raycast](https://www.raycast.com) on macOS. That extension is available in the [raycast-elgato-light](https://github.com/wassimk/raycast-elgato-light) repository.

### Usage

The CLI tool has my Elgato light IP address of *192.168.0.25*, preferred brightness, and temperature hard-coded as the defaults.

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

Use a non-default IP address for the light on any command.

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
