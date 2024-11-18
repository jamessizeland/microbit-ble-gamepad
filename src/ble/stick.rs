use defmt::info;
use embassy_time::{Duration, Timer};
use microbit_bsp::{
    ble::SoftdeviceError,
    embassy_nrf::{
        interrupt::{self, InterruptExt as _},
        peripherals::{P0_03, P0_04, SAADC},
        saadc::{self, Input as _, Saadc},
    },
};
use trouble_host::prelude::*;

use crate::io::{
    display::{AsyncDisplay, DisplayFrame},
    Irqs,
};

use super::BleServer;

#[gatt_service(uuid = "7e701cf1-b1df-42a1-bb5f-6a1028c793b0")]
pub struct StickService {
    #[characteristic(uuid = "e3d1afe4-b414-44e3-be54-0ea26c394eba", read, notify)]
    x: i8,
    #[characteristic(uuid = "65133212-952b-4000-a735-ea558db3ca7b", read, notify)]
    y: i8,
}

pub fn init_analog_adc(x_pin: P0_03, y_pin: P0_04, adc: SAADC) -> Saadc<'static, 2> {
    let config = saadc::Config::default();
    interrupt::SAADC.set_priority(interrupt::Priority::P3);
    let channel_cfg = saadc::ChannelConfig::single_ended(x_pin.degrade_saadc());
    let channel_cfg2 = saadc::ChannelConfig::single_ended(y_pin.degrade_saadc());
    saadc::Saadc::new(adc, Irqs, config, [channel_cfg, channel_cfg2])
}

struct Axis {
    offset: i16,
    divider: i16,
    old: i8,
}

impl Axis {
    fn new(offset: i16, divider: i16) -> Self {
        Self {
            offset,
            divider,
            old: 0,
        }
    }
    fn changed(&mut self, new_raw: i16) -> Option<i8> {
        let new = -((new_raw - self.offset) / self.divider) as i8; // invert the value
        if new != self.old {
            self.old = new;
            Some(new as i8)
        } else {
            None
        }
    }
}

pub async fn analog_stick_task(
    server: &BleServer<'_>,
    conn: &Connection<'_>,
    saadc: &mut Saadc<'_, 2>,
    display: &AsyncDisplay,
) -> Result<(), BleHostError<SoftdeviceError>> {
    let debounce = Duration::from_millis(20);
    info!("analog stick service online");
    let mut buf = [0i16; 2];
    saadc.calibrate().await;
    // full range around 3740, so divide by 935 to get a range of -3, -2, -1, 0, 1, 2, 3
    let offset = 3740 / 2;
    let divider = 623;
    let mut x_axis = Axis::new(offset, divider);
    let mut y_axis = Axis::new(offset, divider);
    loop {
        // read adc values for x and y, and if they have changed by a certain amount, notify
        // we are reducing the number of analogue stick levels to a range of -2 to 2
        saadc.sample(&mut buf).await;
        // display the x and y values on the led matrix
        if let Some(x) = x_axis.changed(buf[0]) {
            server.notify(&server.stick.x, conn, &x).await?;
        }
        if let Some(y) = y_axis.changed(buf[1]) {
            server.notify(&server.stick.y, conn, &y).await?;
        }
        if !(x_axis.old == 0 && y_axis.old == 0) {
            // only display if the stick is not centered
            display
                .display(
                    DisplayFrame::Coord {
                        x: x_axis.old,
                        y: y_axis.old,
                    },
                    Duration::from_millis(20),
                )
                .await;
        }
        Timer::after(debounce).await;
    }
}
