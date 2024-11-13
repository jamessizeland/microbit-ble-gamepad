pub mod advertiser;
pub mod gatt;
pub mod hid;
pub mod stick;

use microbit_bsp::ble::{MultiprotocolServiceLayer, SoftdeviceController};
use trouble_host::{prelude::*, HostResources, Peripheral};

/// Size of L2CAP packets (ATT MTU is this - 4)
const L2CAP_MTU: usize = 251;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

pub type Controller = SoftdeviceController<'static>;

pub type SdcResources<'d> =
    HostResources<Controller, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU>;

pub type SdcPeripheral<'d> = Peripheral<'d, Controller>;

#[embassy_executor::task]
pub async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await;
}

#[embassy_executor::task]
async fn ble_task(mut runner: Runner<'static, SoftdeviceController<'static>>) {
    runner.run().await.expect("Error in BLE task");
}
