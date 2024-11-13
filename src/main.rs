#![no_std]
#![no_main]

mod ble;
mod io;

use defmt_rtt as _;
use panic_probe as _;

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::Duration;
use microbit_bsp::{display::Brightness, embassy_nrf::gpio::Pin as _, Microbit};

use crate::{
    ble::{
        gatt::{gatt_server_task, GamepadServer},
        hid::{buttons_task, GamepadInputs},
        stick::{analog_stick_task, init_analog_adc},
    },
    io::{
        audio::{AsyncAudio, Tune},
        display::{AsyncDisplay, DisplayFrame::*},
        to_button,
    },
};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Hello World!");
    let name = "Rust Gamepad";
    let board = Microbit::new(Default::default());

    // Spawn Async Embassy Tasks
    let display = AsyncDisplay::new(spawner, board.display);
    let speaker = AsyncAudio::new(spawner, board.pwm0, board.speaker);
    let (sdc, mpsl) = board
        .ble
        .init(board.timer0, board.rng)
        .expect("BLE stack failed to initialize");
    let (server, mut advertiser) =
        GamepadServer::start_gatt(name, spawner, sdc, mpsl).expect("Failed to start GATT server");

    let mut gamepad_buttons = GamepadInputs::new(
        server,
        board.btn_a,
        board.btn_b,
        to_button(board.p12.degrade()),
        to_button(board.p13.degrade()),
        to_button(board.p14.degrade()),
        to_button(board.p15.degrade()),
    );

    let mut analog_stick = init_analog_adc(board.p1, board.p2, board.saadc);

    display.set_brightness(Brightness::MAX).await;
    display.scroll("BLE!").await;

    // Main loop
    loop {
        display.display(QuestionMark, Duration::from_secs(2)).await;
        // advertise for connections
        if let Ok(conn) = advertiser.advertise().await {
            let pause = Duration::from_secs(1);
            speaker.play_tune(Tune::Connect).await;
            display.display_blocking(Heart, pause).await;

            let gatt = gatt_server_task(server, &conn);
            let buttons = buttons_task(&mut gamepad_buttons, &conn, &display);
            let analog = analog_stick_task(server, &conn, &mut analog_stick, &display);
            // futures::pin_mut!(gatt, buttons, analog);
            embassy_futures::select::select3(gatt, buttons, analog).await;
            speaker.play_tune(Tune::Disconnect).await;
        }
    }
}
