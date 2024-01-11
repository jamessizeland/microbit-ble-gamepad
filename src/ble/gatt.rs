use defmt::info;
use embassy_nrf::saadc::Saadc;
use embassy_time::{Duration, Timer};
use nrf_softdevice::ble::{gatt_server, Connection};

#[nrf_softdevice::gatt_server]
pub struct GamepadServer {
    bas: BatteryService,
}

#[nrf_softdevice::gatt_service(uuid = "180f")]
pub struct BatteryService {
    #[characteristic(uuid = "2a19", read, notify)]
    battery_level: i16,
}

/// A BLE GATT server
pub async fn gatt_server_task(server: &GamepadServer, conn: &Connection) {
    gatt_server::run(&conn, server, |e| match e {
        GamepadServerEvent::Bas(e) => match e {
            BatteryServiceEvent::BatteryLevelCccdWrite { notifications } => {
                defmt::info!("battery notifications: {}", notifications)
            }
        },
    })
    .await;
    info!("Gatt server task finished");
}

/// Notify the battery level every 60 seconds
pub async fn notify_battery_level(
    server: &GamepadServer,
    connection: &Connection,
    saadc: &mut Saadc<'_, 1>,
) {
    info!("Notifying battery level");
    loop {
        let mut buf = [0i16; 1];
        saadc.sample(&mut buf).await;
        let adc_raw = buf[0];

        match server.bas.battery_level_notify(connection, &adc_raw) {
            Ok(_) => info!("Battery level notified"),
            Err(_) => defmt::unwrap!(server.bas.battery_level_set(&adc_raw)),
        };
        Timer::after(Duration::from_secs(60)).await;
    }
}
