use crate::ble::{advertiser, enable_softdevice, softdevice_task};

use super::advertiser::Advertiser;
// use super::battery::*;
use super::hid::*;
use super::stick::*;
use defmt::info;
use embassy_executor::Spawner;
use nrf_softdevice::ble::{gatt_server, Connection};
use static_cell::StaticCell;

#[nrf_softdevice::gatt_server]
pub struct GamepadServer {
    // pub bas: BatteryService,
    pub hid: ButtonService,
    pub stick: StickService,
}

impl GamepadServer {
    pub fn start_gatt(
        name: &'static str,
        spawner: Spawner,
    ) -> (&'static GamepadServer, Advertiser) {
        // Spawn the underlying softdevice task
        let sd = enable_softdevice(name);
        // Create a BLE GATT server and make it static
        static SERVER: StaticCell<GamepadServer> = StaticCell::new();
        let server = SERVER.init(GamepadServer::new(sd).unwrap());
        info!("Starting Gatt Server");
        defmt::unwrap!(spawner.spawn(softdevice_task(sd)));
        let advertiser = advertiser::AdvertiserBuilder::new(name, sd).build();
        (server, advertiser)
    }
}

/// A BLE GATT server
pub async fn gatt_server_task(server: &GamepadServer, conn: &Connection) {
    gatt_server::run(&conn, server, |e| match e {
        // GamepadServerEvent::Bas(e) => match e {
        //     BatteryServiceEvent::BatteryLevelCccdWrite { notifications } => {
        //         defmt::info!("battery notifications: {}", notifications)
        //     }
        // },
        GamepadServerEvent::Hid(e) => match e {
            ButtonServiceEvent::ButtonACccdWrite { notifications }
            | ButtonServiceEvent::ButtonBCccdWrite { notifications }
            | ButtonServiceEvent::ButtonCCccdWrite { notifications }
            | ButtonServiceEvent::ButtonDCccdWrite { notifications }
            | ButtonServiceEvent::ButtonECccdWrite { notifications }
            | ButtonServiceEvent::ButtonFCccdWrite { notifications } => {
                info!("button: {}", notifications)
            }
        },
        GamepadServerEvent::Stick(e) => match e {
            StickServiceEvent::XCccdWrite { notifications }
            | StickServiceEvent::YCccdWrite { notifications } => {
                info!("stick: {}", notifications);
            }
        },
    })
    .await;
    info!("Gatt server task finished");
}
