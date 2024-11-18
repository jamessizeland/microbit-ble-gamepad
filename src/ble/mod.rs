pub mod advertiser;
pub mod gatt;
pub mod hid;
pub mod stick;

use microbit_bsp::ble::{MultiprotocolServiceLayer, SoftdeviceController};
use trouble_host::prelude::*;

/// Size of L2CAP packets (ATT MTU is this - 4)
const L2CAP_MTU: usize = 251;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

pub type BleServer<'d> = gatt::Server<'d, 'd, SoftdeviceController<'d>>;

pub type BleController = SoftdeviceController<'static>;

pub type BleResources =
    HostResources<BleController, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU>;

#[embassy_executor::task]
pub async fn mpsl_task(mpsl: &'static MultiprotocolServiceLayer<'static>) -> ! {
    mpsl.run().await;
}

#[embassy_executor::task]
async fn ble_task(mut runner: Runner<'static, BleController>) {
    runner.run().await.expect("Error in BLE task");
}
