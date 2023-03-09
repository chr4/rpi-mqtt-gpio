# rpi-mqtt-gpio

Exposes general purpose outputs (GPIO) to an MQTT server (read/write). Currently written for the Raspberry Pi.

![build](https://github.com/chr4/rpi-mqtt-gpio/workflows/build/badge.svg)


This project is heavily inspired by [mqtt-io](https://github.com/flyte/mqtt-io) - written in Python, which supports much more hardware.

I've written this because I needed:

1. Something that's better shippable in [NixOS](https://nixos.org/) (Python dependencies are a mess)
2. I was struggling with [reconnect issues](https://github.com/flyte/mqtt-io/issues/282)
3. Rewrite it in Rust (tm)!


## Installation

You can get a binary for armv7 from Github.


## Build

```bash
# Compile
cargo build --release

# Cross compile to arm (Raspberry Pi 3)
cargo install cross --git https://github.com/cross-rs/cross
cross build --target armv7-unknown-linux-musleabihf --release
```


## Configuration

```yaml
mqtt:
  host: 192.168.9.2

digital_outputs:
  # MQTT topics configured here are compatible with mqtt-io
  - gpio: 5
    mqtt_topic: "heatpump/output/i1"
    mqtt_topic_set: "heatpump/output/i1/set"

  - gpio: 22
    mqtt_topic: "heatpump/output/i3"
    mqtt_topic_set: "heatpump/output/i3/set"
```

See [config.yaml](config.yaml) in this repository for an example with all available options.


## Usage

```bash
rpi-mqtt-gpio <config.yaml>
```
