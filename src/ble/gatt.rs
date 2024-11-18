use super::advertiser::{Advertiser, AdvertiserBuilder};
use super::{ble_task, mpsl_task, BleResources};
use super::{hid::*, BleServer};
use super::{stick::*, BleController};
use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_futures::select::Either;
use microbit_bsp::ble::{MultiprotocolServiceLayer, SoftdeviceError};
use static_cell::StaticCell;
use trouble_host::prelude::*;
use trouble_host::types::gatt_traits::GattValue;

/// Allow a central to decide which player this controller belongs to
#[gatt_service(uuid = "8f701cf1-b1df-42a1-bb5f-6a1028c793b0")]
pub struct Player {
    #[characteristic(uuid = "e3d1afe4-b414-44e3-be54-0ea26c394eba", read, write, on_write = on_write)]
    index: u8,
}

fn on_write(_: &Connection<'_>, value: &[u8]) -> Result<(), ()> {
    if let Ok(index) = u8::from_gatt(value) {
        info!("Player index set to {:?}", index);
    };
    Ok(())
}

#[gatt_server(attribute_data_size = 100)]
pub struct Server {
    // pub bas: BatteryService,
    pub hid: ButtonService,
    pub stick: StickService,
    pub player: Player,
}

impl Server<'static, 'static, BleController> {
    pub fn start_gatt(
        name: &'static str,
        spawner: Spawner,
        controller: BleController,
        mpsl: &'static MultiprotocolServiceLayer<'static>,
    ) -> Result<(&'static Self, Advertiser<'static, BleController>), BleHostError<SoftdeviceError>>
    {
        spawner.must_spawn(mpsl_task(mpsl));

        let address = Address::random([0x41, 0x5A, 0xE3, 0x1E, 0x83, 0xE7]);
        info!("Our address = {:?}", address);

        let resources = {
            static RESOURCES: StaticCell<BleResources> = StaticCell::new();
            RESOURCES.init(BleResources::new(PacketQos::None))
        };
        let (stack, peripheral, _, runner) = trouble_host::new(controller, resources)
            .set_random_address(address)
            .build();
        let server = {
            static SERVER: StaticCell<BleServer<'_>> = StaticCell::new();
            SERVER.init(
                Server::new_with_config(
                    stack,
                    GapConfig::Peripheral(PeripheralConfig {
                        name,
                        appearance: &appearance::GAMEPAD,
                    }),
                )
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
    loop {
        if let Either::First(event) = select(conn.next(), server.run()).await {
            match event {
                ConnectionEvent::Disconnected { reason } => {
                    info!("[gatt] Disconnected: {:?}", reason);
                    break;
                }
                ConnectionEvent::Gatt { event, .. } => match event {
                    GattEvent::Read { value_handle } => {
                        info!("[gatt] Server Write event on {:?}", value_handle);
                    }
                    GattEvent::Write { value_handle } => {
                        info!("[gatt] Read event on {:?}", value_handle);
                    }
                },
            }
        }
    }
    info!("Gatt server task finished");
}
