# Rust BLE Embedded Gamepad

## Prerequisites

* [BBC micro:bit v2](https://microbit.org/)

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
