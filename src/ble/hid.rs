use defmt::info;
use embassy_futures::select;
use embassy_time::{Duration, Timer};
use microbit_bsp::{ble::SoftdeviceError, Button};
use trouble_host::prelude::*;

use crate::io::display::{self, DisplayFrame};

use super::gatt::GamepadServer;

#[gatt_service(uuid = "260279e7-a5dd-447b-9bd8-e624ef464d6e")]
pub struct ButtonService {
    #[characteristic(uuid = "c665eb11-eee4-452b-9047-a98a3916bd80", read, notify)]
    button_a: bool,
    #[characteristic(uuid = "7c9a1a08-ecf2-4f7d-a24b-0ab01615cc77", read, notify)]
    button_b: bool,
    #[characteristic(uuid = "163a7681-4b8b-4249-899d-ae1a634ce692", read, notify)]
    button_c: bool,
    #[characteristic(uuid = "c8ede9b0-4eeb-4f31-b8d4-f920881961fa", read, notify)]
    button_d: bool,
    #[characteristic(uuid = "7729d82d-a8b9-4c3e-95bf-3794b70aba56", read, notify)]
    button_e: bool,
    #[characteristic(uuid = "f8f17954-f235-4d71-8ece-1522ec067c55", read, notify)]
    button_f: bool,
}

/// A struct containing a button and its corresponding characteristic handle
pub struct GamepadButton {
    pub name: char,
    /// The pin that the button is connected to
    pub input: Button,
    /// The handle of the button's characteristic
    pub ble_handle: Characteristic<bool>,
}

/// Notify when this button is pressed or released
pub async fn notify_button_state(
    button: &mut GamepadButton,
    connection: &Connection<'_>,
    display: &display::AsyncDisplay,
    server: &GamepadServer<'_>,
) -> Result<(), BleHostError<SoftdeviceError>> {
    let debounce = Duration::from_millis(50);
    info!("button {} service online", button.name);
    loop {
        button.input.wait_for_low().await;
        info!("button {} pressed", button.name);
        server.notify(&button.ble_handle, connection, &true).await?;
        display
            .display(
                DisplayFrame::Letter(button.name),
                Duration::from_millis(200),
            )
            .await;
        Timer::after(debounce).await;
        button.input.wait_for_high().await;
        info!("button {} released", button.name);
        server
            .notify(&button.ble_handle, connection, &false)
            .await?;
        Timer::after(debounce).await;
    }
}

pub async fn buttons_task(
    buttons: &mut GamepadInputs,
    conn: &Connection<'_>,
    display: &display::AsyncDisplay,
) {
    let futures = [
        notify_button_state(&mut buttons.b, conn, display, buttons.server),
        notify_button_state(&mut buttons.a, conn, display, buttons.server),
        notify_button_state(&mut buttons.c, conn, display, buttons.server),
        notify_button_state(&mut buttons.d, conn, display, buttons.server),
        notify_button_state(&mut buttons.e, conn, display, buttons.server),
        notify_button_state(&mut buttons.f, conn, display, buttons.server),
    ];
    let _ = select::select_array(futures).await;
}

impl GamepadButton {
    /// Create a new button with the given pin and characteristic handle
    pub fn new(name: char, input: Button, ble_handle: Characteristic<bool>) -> Self {
        info!("button {} created {}", name, ble_handle);
        Self {
            name,
            input,
            ble_handle,
        }
    }
}

/// A struct containing all of the buttons on the microbit
pub struct GamepadInputs {
    server: &'static GamepadServer<'static>,
    pub a: GamepadButton,
    pub b: GamepadButton,
    pub c: GamepadButton,
    pub d: GamepadButton,
    pub e: GamepadButton,
    pub f: GamepadButton,
}

impl GamepadInputs {
    /// Create a new GamepadInputs struct with the given pins
    pub fn new(
        server: &'static GamepadServer<'_>,
        a: Button,
        b: Button,
        c: Button,
        d: Button,
        e: Button,
        f: Button,
    ) -> Self {
        Self {
            server,
            a: GamepadButton::new('A', a, server.hid.button_a),
            b: GamepadButton::new('B', b, server.hid.button_b),
            c: GamepadButton::new('C', c, server.hid.button_c),
            d: GamepadButton::new('D', d, server.hid.button_d),
            e: GamepadButton::new('E', e, server.hid.button_e),
            f: GamepadButton::new('F', f, server.hid.button_f),
        }
    }
}
