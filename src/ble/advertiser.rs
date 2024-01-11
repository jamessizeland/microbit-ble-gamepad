use defmt::info;
use heapless::Vec;
use nrf_softdevice::{
    ble::{
        peripheral::{self, AdvertiseError},
        Connection,
    },
    raw, Softdevice,
};

/// BLE advertiser
pub struct AdvertiserBuilder<'a> {
    /// Name of the device
    name: &'a str,
}

pub struct Advertiser {
    advertiser_data: Vec<u8, 31>,
    scan_data: [u8; 4],
}

/// A BLE advertiser
impl<'a> AdvertiserBuilder<'a> {
    /// Create a new advertiser builder
    pub fn new(name: &'a str) -> Self {
        Self { name }
    }
    /// Build the advertiser
    pub fn build(self) -> Advertiser {
        let name: &str;
        if self.name.len() > 22 {
            name = &self.name[..22];
            info!("Name truncated to {}", name);
        } else {
            name = self.name;
        }
        let mut advertiser_data = Vec::new();
        #[rustfmt::skip]
        advertiser_data.extend_from_slice(&[
            0x02, raw::BLE_GAP_AD_TYPE_FLAGS as u8, raw::BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE as u8,
            0x03, raw::BLE_GAP_AD_TYPE_16BIT_SERVICE_UUID_MORE_AVAILABLE as u8, 0x09, 0x18,
            (1 + name.len() as u8), raw::BLE_GAP_AD_TYPE_COMPLETE_LOCAL_NAME as u8]).unwrap();

        advertiser_data
            .extend_from_slice(name.as_bytes())
            .ok()
            .unwrap();
        #[rustfmt::skip]
        let scan_data = [
            0x03, raw::BLE_GAP_AD_TYPE_16BIT_SERVICE_UUID_MORE_AVAILABLE as u8, 0x09, 0x18,
        ];
        Advertiser {
            advertiser_data,
            scan_data,
        }
    }
}

impl Advertiser {
    /// Advertise and connect to a device with the given name
    pub async fn advertise(&self, sd: &Softdevice) -> Result<Connection, AdvertiseError> {
        let config = peripheral::Config::default();
        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
            adv_data: &self.advertiser_data[..],
            scan_data: &self.scan_data[..],
        };
        info!("advertising");
        let conn = peripheral::advertise_connectable(sd, adv, &config).await;
        info!("connection established");
        conn
    }
}
