use microbit_bsp::embassy_nrf::{
    bind_interrupts,
    gpio::{AnyPin, Input, Pull},
    interrupt::{self, InterruptExt as _},
    peripherals,
    saadc::{self, Input as _, Saadc},
    Peripheral,
};

pub mod audio;
pub mod display;

pub fn input_pin(pin: AnyPin) -> Input<'static, AnyPin> {
    Input::new(pin, Pull::Up)
}

bind_interrupts!(pub struct Irqs {
    SAADC => saadc::InterruptHandler;
});

/// Initialize the analog digital converter pins
pub fn init_adc<'d>(
    pin: saadc::AnyInput,
    adc: impl Peripheral<P = peripherals::SAADC> + 'd,
) -> Saadc<'d, 1> {
    let config = saadc::Config::default();
    interrupt::SAADC.set_priority(interrupt::Priority::P3);
    let channel_cfg = saadc::ChannelConfig::single_ended(pin.degrade_saadc());
    saadc::Saadc::new(adc, Irqs, config, [channel_cfg])
}

pub fn to_button(pin: AnyPin) -> Input<'static, AnyPin> {
    Input::new(pin, Pull::Up)
}
