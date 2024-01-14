# Rust BLE Embedded Gamepad

## Overview

Demo of bringing up Bluetooth Low Energy (BLE) using Embedded Rust, by turning a BBC Microbit ([nordic NRF52833](https://www.nordicsemi.com/products/nrf52833)) into a wireless game controller.

## Prerequisites

* [BBC micro:bit v2](https://microbit.org/)
* [Gamepad Adapter](https://www.amazon.co.uk/ELECFREAKS-microbit-Joystick-Wireless-Control/dp/B09Q17XZ1N/)

![controller](./img/gamepad.jpg)

## Setup

Flash the softdevice onto the micro:bit (only needed the first time you run it):

```bash
cargo install probe-rs --features cli
probe-rs-cli erase --chip nrf52833
probe-rs-cli download softdevice/s140_nrf52_7.3.0_softdevice.hex --format hex --chip nRF52833_xxAA
```

```bash
cargo run --release
```
