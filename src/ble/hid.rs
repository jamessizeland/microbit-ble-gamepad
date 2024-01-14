use defmt::info;
use embassy_time::{Duration, Timer};
use microbit_bsp::Button;
use nrf_softdevice::ble::{gatt_server::notify_value, Connection};

use crate::io::display::{self, DisplayFrame};

#[nrf_softdevice::gatt_service(uuid = "1812")]
pub struct ButtonService {
    #[characteristic(uuid = "2ae2", read, notify)]
    button_a: bool,
    #[characteristic(uuid = "2ae2", read, notify)]
    button_b: bool,
    #[characteristic(uuid = "2ae2", read, notify)]
    button_c: bool,
    #[characteristic(uuid = "2ae2", read, notify)]
    button_d: bool,
    #[characteristic(uuid = "2ae2", read, notify)]
    button_e: bool,
    #[characteristic(uuid = "2ae2", read, notify)]
    button_f: bool,
}

/// A struct containing a button and its corresponding characteristic handle
pub struct GamepadButton {
    pub name: char,
    /// The pin that the button is connected to
    pub input: Button,
    /// The handle of the button's characteristic
    pub ble_handle: u16,
}

/// Notify when this button is pressed or released
pub async fn notify_button_state(
    button: &mut GamepadButton,
    connection: &Connection,
    display: &display::AsyncDisplay,
) {
    let debounce = Duration::from_millis(50);
    info!("button {} service online", button.name);
    loop {
        button.input.wait_for_low().await;
        info!("button {} pressed", button.name);
        notify_value(connection, button.ble_handle, &[0x01]).ok();
        display
            .display(
                DisplayFrame::Letter(button.name),
                Duration::from_millis(200),
            )
            .await;
        Timer::after(debounce).await;
        button.input.wait_for_high().await;
        info!("button {} released", button.name);
        notify_value(connection, button.ble_handle, &[0x00]).ok();
        Timer::after(debounce).await;
    }
}

pub async fn buttons_task(
    buttons: &mut GamepadInputs,
    conn: &Connection,
    display: &display::AsyncDisplay,
) {
    let futures = [
        notify_button_state(&mut buttons.b, conn, display),
        notify_button_state(&mut buttons.a, conn, display),
        notify_button_state(&mut buttons.c, conn, display),
        notify_button_state(&mut buttons.d, conn, display),
        notify_button_state(&mut buttons.e, conn, display),
        notify_button_state(&mut buttons.f, conn, display),
    ];
    embassy_futures::select::select_array(futures).await;
}

impl GamepadButton {
    /// Create a new button with the given pin and characteristic handle
    pub fn new(name: char, input: Button, ble_handle: u16) -> Self {
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
    pub a: GamepadButton,
    pub b: GamepadButton,
    pub c: GamepadButton,
    pub d: GamepadButton,
    pub e: GamepadButton,
    pub f: GamepadButton,
    // analog
}

impl GamepadInputs {
    /// Create a new GamepadInputs struct with the given pins
    pub fn new(
        gamepad_service: &ButtonService,
        a: Button,
        b: Button,
        c: Button,
        d: Button,
        e: Button,
        f: Button,
    ) -> Self {
        Self {
            a: GamepadButton::new('A', a, gamepad_service.button_a_value_handle),
            b: GamepadButton::new('B', b, gamepad_service.button_b_value_handle),
            c: GamepadButton::new('C', c, gamepad_service.button_c_value_handle),
            d: GamepadButton::new('D', d, gamepad_service.button_d_value_handle),
            e: GamepadButton::new('E', e, gamepad_service.button_e_value_handle),
            f: GamepadButton::new('F', f, gamepad_service.button_f_value_handle),
        }
    }
}
