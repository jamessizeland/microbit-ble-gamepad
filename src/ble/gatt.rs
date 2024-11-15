use super::{
    advertiser::{Advertiser, AdvertiserBuilder},
    ble_task,
    hid::*,
    mpsl_task,
    stick::*,
    SdcResources,
};
use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use microbit_bsp::ble::{MultiprotocolServiceLayer, SoftdeviceController};
use static_cell::StaticCell;
use trouble_host::prelude::*;

pub type GamepadServer<'d> = Server<'d, 'd, SoftdeviceController<'d>>;

#[gatt_service(uuid = "460279e7-a5dd-447b-9bd8-e624ef464d6e")]
pub struct Mode {
    #[characteristic(uuid = "f8f17959-f235-4d71-8ece-1522ec067c55", read, write)]
    pub mode: u8,
}

#[gatt_server(attribute_data_size = 100)]
pub struct Server {
    // pub bas: BatteryService,
    pub hid: ButtonService,
    pub stick: StickService,
    pub mode: Mode,
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
                    appearance: &appearance::GAMEPAD,
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
        match select(conn.event(), server.next()).await {
            Either::First(event) => {
                if let ConnectionEvent::Disconnected { reason } = event {
                    info!("[gatt] Disconnected: {:?}", reason);
                    break;
                }
            }
            Either::Second(event) => match event {
                Ok(GattEvent::Write {
                    value_handle,
                    connection: _,
                }) => {
                    info!("[gatt] Server Write event on {:?}", value_handle);
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
