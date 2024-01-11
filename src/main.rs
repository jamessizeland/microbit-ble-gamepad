#![no_std]
#![no_main]

mod ble;
mod board;

use board::{init_adc, Microbit};
use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::{config::Config, interrupt::Priority, saadc::Input};
use futures::pin_mut;
use static_cell::StaticCell;

use defmt_rtt as _;
use panic_probe as _;

use crate::{
    ble::{advertiser, enable_softdevice, gatt::*, softdevice_task},
    board::display,
};

// Application must run at a lower priority than softdevice
fn config() -> Config {
    let mut config = Config::default();
    config.gpiote_interrupt_priority = Priority::P2;
    config.time_interrupt_priority = Priority::P2;
    config
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");
    let name = "Embassy Microbit";
    let board = Microbit::new(config());
    let mut saadc = init_adc(board.adc_pin.degrade_saadc(), board.saadc);

    // Spawn the underlying softdevice task
    let sd = enable_softdevice(name);
    // Create a BLE GATT server and make it static
    static SERVER: StaticCell<GamepadServer> = StaticCell::new();
    let server = SERVER.init(GamepadServer::new(sd).unwrap());
    info!("Starting Server");

    let advertiser = advertiser::AdvertiserBuilder::new(name).build();
    info!("Built Advertiser");
    let mut display = board.display;
    display.set_brightness(display::Brightness::MAX);
    display.scroll("BLE!").await;

    defmt::unwrap!(spawner.spawn(softdevice_task(sd)));
    loop {
        display.scroll("Searching").await;
        let conn = defmt::unwrap!(advertiser.advertise(sd).await); // advertise for connections
        display.scroll("Connected").await;
        let gatt = gatt_server_task(server, &conn);
        let bas = notify_battery_level(server, &conn, &mut saadc);
        pin_mut!(gatt, bas);
        embassy_futures::select::select(gatt, bas).await;
    }
}
