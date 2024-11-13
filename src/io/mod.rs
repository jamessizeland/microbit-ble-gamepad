pub mod audio;
pub mod display;

use microbit_bsp::embassy_nrf::{
    bind_interrupts,
    gpio::{AnyPin, Input, Pull},
    saadc,
};

bind_interrupts!(pub struct Irqs {
    SAADC => saadc::InterruptHandler;
});

pub fn to_button(pin: AnyPin) -> Input<'static> {
    Input::new(pin, Pull::Up)
}
