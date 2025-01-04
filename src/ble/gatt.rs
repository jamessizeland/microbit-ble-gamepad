use super::advertiser::{Advertiser, AdvertiserBuilder};
use super::hid::*;
use super::{ble_task, mpsl_task, BleResources};
use super::{stick::*, BleController};
use defmt::{info, warn};
use embassy_executor::Spawner;
use microbit_bsp::ble::{MultiprotocolServiceLayer, SoftdeviceError};
use static_cell::StaticCell;
use trouble_host::prelude::*;
use trouble_host::types::gatt_traits::GattValue;

/// Allow a central to decide which player this controller belongs to
#[gatt_service(uuid = "8f701cf1-b1df-42a1-bb5f-6a1028c793b0")]
pub struct Player {
    #[characteristic(uuid = "e3d1afe4-b414-44e3-be54-0ea26c394eba", read, write)]
    index: u8,
}

#[gatt_server]
pub struct BleServer {
    // pub bas: BatteryService,
    pub hid: ButtonService,
    pub stick: StickService,
    pub player: Player,
}

impl BleServer<'static> {
    pub fn start_gatt(
        name: &'static str,
        spawner: Spawner,
        controller: BleController,
        mpsl: &'static MultiprotocolServiceLayer<'static>,
    ) -> Result<(&'static Self, Advertiser<'static, BleController>), BleHostError<SoftdeviceError>>
    {
        spawner.must_spawn(mpsl_task(mpsl));

        let address = Address::random([0x42, 0x5A, 0xE3, 0x1E, 0x83, 0xE7]);
        info!("Our address = {:?}", address);

        let resources = {
            static RESOURCES: StaticCell<BleResources> = StaticCell::new();
            RESOURCES.init(BleResources::new(PacketQos::None))
        };
        let (_, peripheral, _, runner) = trouble_host::new(controller, resources)
            .set_random_address(address)
            .build();
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
        spawner.must_spawn(ble_task(runner));
        let advertiser = AdvertiserBuilder::new(name, peripheral).build()?;
        Ok((server, advertiser))
    }
}

/// A BLE GATT server
pub async fn gatt_server_task(server: &BleServer<'_>, conn: &Connection<'_>) {
    let index = server.player.index;
    loop {
        match conn.next().await {
            ConnectionEvent::Disconnected { reason } => {
                info!("[gatt] disconnected: {:?}", reason);
                break;
            }
            ConnectionEvent::Gatt { data } => match data.process(server).await {
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
            },
        }
    }
    info!("Gatt server task finished");
}
