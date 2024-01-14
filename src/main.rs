#![no_std]
#![no_main]

mod ble;
mod io;

use defmt::info;
use embassy_executor::Spawner;
use embassy_nrf::{config::Config, gpio::Pin, interrupt::Priority};
use embassy_time::Duration;
use microbit_bsp::{display::Brightness, Microbit};
use static_cell::StaticCell;

use defmt_rtt as _;
use panic_probe as _;

use crate::{
    ble::{
        advertiser, enable_softdevice,
        gatt::*,
        hid::{buttons_task, GamepadInputs},
        softdevice_task,
    },
    io::{
        audio::{AsyncAudio, Tune},
        display::{AsyncDisplay, DisplayFrame::*},
        to_button,
    },
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
    let name = "Microbit Gamepad";
    let board = Microbit::new(config());
    // let mut saadc = init_adc(.degrade_saadc(), board.saadc);

    let display = AsyncDisplay::new(spawner, board.display);
    let speaker = AsyncAudio::new(spawner, board.pwm0, board.speaker);

    // Spawn the underlying softdevice task
    let sd = enable_softdevice(name);
    // Create a BLE GATT server and make it static
    static SERVER: StaticCell<GamepadServer> = StaticCell::new();
    let server = SERVER.init(GamepadServer::new(sd).unwrap());
    info!("Starting Gatt Server");
    defmt::unwrap!(spawner.spawn(softdevice_task(sd)));
    let advertiser = advertiser::AdvertiserBuilder::new(name).build();

    let mut gamepad_buttons = GamepadInputs::new(
        &server.hid,
        board.btn_a,
        board.btn_b,
        to_button(board.p12.degrade()),
        to_button(board.p13.degrade()),
        to_button(board.p14.degrade()),
        to_button(board.p15.degrade()),
    );

    display.set_brightness(Brightness::MAX).await;
    display.scroll("BLE!").await;

    // Main loop
    loop {
        display.display(QuestionMark, Duration::from_secs(2)).await;
        let conn = defmt::unwrap!(advertiser.advertise(sd).await); // advertise for connections
        display.display(Heart, Duration::from_secs(2)).await;
        speaker.play_tune(Tune::Connect).await;
        let gatt = gatt_server_task(server, &conn);
        let buttons = buttons_task(&mut gamepad_buttons, &conn);
        futures::pin_mut!(gatt, buttons);
        embassy_futures::select::select(gatt, buttons).await;
        speaker.play_tune(Tune::Disconnect).await;
    }
}
