use super::battery::*;
use super::hid::*;
use defmt::info;
use nrf_softdevice::ble::{gatt_server, Connection};

#[nrf_softdevice::gatt_server]
pub struct GamepadServer {
    // pub bas: BatteryService,
    pub hid: ButtonService,
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
    })
    .await;
    info!("Gatt server task finished");
}
