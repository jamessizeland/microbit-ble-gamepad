use super::hid::*;
use super::{ble_task, mpsl_task, BleResources};
use super::{stick::*, BleController};
use defmt::{info, warn};
use embassy_executor::Spawner;
use microbit_bsp::ble::{MultiprotocolServiceLayer, SoftdeviceController, SoftdeviceError};
use static_cell::StaticCell;
use trouble_host::prelude::*;

/// Allow a central to decide which player this controller belongs to
#[gatt_service(uuid = "8f701cf1-b1df-42a1-bb5f-6a1028c793b0")]
pub struct Player {
    #[characteristic(uuid = "e3d1afe4-b414-44e3-be54-0ea26c377eba", read, write)]
    index: u8,
}

#[gatt_server]
pub struct BleServer {
    // pub bas: BatteryService,
    pub hid: ButtonService,
    pub stick: StickService,
    pub player: Player,
}

impl<'d> BleServer<'d> {
    /// Build the stack for the GATT server and start background tasks required for the
    /// Softdevice (Noridc's BLE stack) to run.
    pub fn start_gatt(
        name: &'d str,
        spawner: Spawner,
        controller: BleController,
        mpsl: &'static MultiprotocolServiceLayer<'_>,
    ) -> Result<(&'static Self, Peripheral<'d, BleController>), BleHostError<SoftdeviceError>> {
        spawner.must_spawn(mpsl_task(mpsl));

        let address = Address::random([0x42, 0x5A, 0xE3, 0x1E, 0x83, 0xE7]);
        info!("Our address = {:?}", address);

        let resources = {
            static RESOURCES: StaticCell<BleResources> = StaticCell::new();
            RESOURCES.init(BleResources::new())
        };
        let stack = {
            static STACK: StaticCell<Stack<'_, SoftdeviceController<'_>>> = StaticCell::new();
            STACK.init(trouble_host::new(controller, resources).set_random_address(address))
        };
        let host = stack.build();
        let server = {
            static SERVER: StaticCell<BleServer<'_>> = StaticCell::new();
            SERVER.init(
                BleServer::new_with_config(GapConfig::Peripheral(PeripheralConfig {
                    name,
                    appearance: &appearance::human_interface_device::GAMEPAD,
                }))
                .expect("Error creating Gatt Server"),
            )
        };
        info!("Starting Gatt Server");
        spawner.must_spawn(ble_task(host.runner));
        Ok((server, host.peripheral))
    }
}

/// A BLE GATT server.
///
/// This is where we can interact with events from the GATT server.
/// This task will run until the connection is disconnected.
pub async fn gatt_server_task(server: &BleServer<'_>, conn: &Connection<'_>) {
    let index = server.player.index;
    loop {
        match conn.next().await {
            ConnectionEvent::Disconnected { reason } => {
                info!("[gatt] Disconnected: {:?}", reason);
                break;
            }
            ConnectionEvent::Gatt { data, .. } => {
                match data.process(server).await {
                    // Server processing emits
                    Ok(Some(GattEvent::Read(event))) => {
                        if event.handle() == index.handle {
                            let value = server.get(&index);
                            info!("[gatt] Read Event to index Characteristic: {:?}", value);
                        }
                    }
                    Ok(Some(GattEvent::Write(event))) => {
                        if event.handle() == index.handle {
                            info!(
                                "[gatt] Write Event to index Characteristic: {:?}",
                                event.data()
                            );
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        warn!("[gatt] error processing event: {:?}", e);
                    }
                }
            }
        }
    }
    info!("Gatt server task finished");
}
