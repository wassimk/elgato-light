# Demo

The README GIF is rendered with [VHS](https://github.com/charmbracelet/vhs) from `elgato-light.tape`.

## Requirements

- `vhs`
- `elgato-light` on your `PATH`
- `python3`
- JetBrainsMono Nerd Font

## Rendering

From the repo root:

```shell
vhs demo/elgato-light.tape
```

This overwrites `demo/elgato-light.gif`.

## How it works

No real lights are touched. `fake_lights.py` serves the Elgato `/elgato/lights` API for two fake lights on `127.0.0.1:19123` and `127.0.0.1:19124`, and writes them to the discovery cache. The tape points `HOME` at a temp directory first, so the CLI reads that cache instead of the real one and skips Bonjour.

Temperatures in the tape (4000K, 5000K) are chosen because they convert to whole mireds, so `status` reports them exactly.
