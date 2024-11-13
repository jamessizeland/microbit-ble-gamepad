use super::advertiser::Advertiser;
use super::hid::*;
use super::stick::*;
use crate::ble::advertiser::AdvertiserBuilder;
use crate::ble::ble_task;
use crate::ble::{mpsl_task, SdcResources};
use defmt::error;
use defmt::info;
use embassy_executor::Spawner;
use embassy_futures::select::select;
use embassy_futures::select::Either;
use embassy_time::Timer;
use microbit_bsp::ble::{MultiprotocolServiceLayer, SoftdeviceController};
use static_cell::StaticCell;
use trouble_host::prelude::*;

pub type GamepadServer<'d> = Server<'d, 'd, SoftdeviceController<'d>>;

#[gatt_server(attribute_data_size = 100)]
pub struct Server {
    // pub bas: BatteryService,
    pub hid: ButtonService,
    pub stick: StickService,
}

impl Server<'static, 'static, SoftdeviceController<'static>> {
    pub fn start_gatt(
        name: &'static str,
        spawner: Spawner,
        sdc: SoftdeviceController<'static>,
        mpsl: &'static MultiprotocolServiceLayer<'static>,
        // ) -> Self {
    ) -> Result<(&'static GamepadServer<'static>, Advertiser<'static>), &'static str> {
        spawner.must_spawn(mpsl_task(mpsl));

        let address = Address::random([0x41, 0x5A, 0xE3, 0x1E, 0x83, 0xE7]);
        info!("Our address = {:?}", address);

        let resources = {
            static RESOURCES: StaticCell<SdcResources<'_>> = StaticCell::new();
            RESOURCES.init(SdcResources::new(PacketQos::None))
        };
        let (stack, peripheral, _, runner) = trouble_host::new(sdc, resources)
            .set_random_address(address)
            .build();
        let server = {
            static SERVER: StaticCell<GamepadServer<'_>> = StaticCell::new();
            SERVER.init(Server::new_with_config(
                stack,
                GapConfig::Peripheral(PeripheralConfig {
                    name,
                    appearance: &appearance::GENERIC_POWER,
                }),
            )?)
        };
        info!("Starting Gatt Server");
        spawner.must_spawn(ble_task(runner));
        let advertiser = AdvertiserBuilder::new(name, peripheral).build().unwrap();
        Ok((server, advertiser))
    }
}

/// A BLE GATT server
pub async fn gatt_server_task(server: &GamepadServer<'_>, conn: &Connection<'_>) {
    // check if the connection is still active every second
    loop {
        match select(Timer::after_secs(1), server.next()).await {
            Either::First(_) => {
                if !conn.is_connected() {
                    break;
                }
            }
            Either::Second(event) => match event {
                Ok(GattEvent::Write {
                    value_handle,
                    connection: _,
                }) => {
                    info!("[gatt] Write event on {:?}", value_handle);
                }
                Ok(GattEvent::Read {
                    value_handle,
                    connection: _,
                }) => {
                    info!("[gatt] Read event on {:?}", value_handle);
                }
                Err(e) => {
                    error!("[gatt] Error processing GATT events: {:?}", e);
                }
            },
        }
    }
    info!("Gatt server task finished");
}
