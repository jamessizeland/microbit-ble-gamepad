use defmt::info;
use heapless::Vec;
use trouble_host::prelude::*;

/// BLE advertiser
pub struct AdvertiserBuilder {
    /// Name of the device
    name: &'static str,
    sd: &'static Softdevice,
}

pub struct Advertiser {
    advertiser_data: Vec<u8, 31>,
    scan_data: [u8; 4],
    sd: &'static Softdevice,
}

/// A BLE advertiser
impl AdvertiserBuilder {
    /// Create a new advertiser builder
    pub fn new(name: &'static str, sd: &'static Softdevice) -> Self {
        Self { name, sd }
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
            sd: self.sd,
        }
    }
}

impl Advertiser {
    /// Advertise and connect to a device with the given name
    pub async fn advertise(&self) -> Connection {
        let config = peripheral::Config::default();
        let adv = peripheral::ConnectableAdvertisement::ScannableUndirected {
            adv_data: &self.advertiser_data[..],
            scan_data: &self.scan_data[..],
        };
        info!("advertising");
        let conn = peripheral::advertise_connectable(self.sd, adv, &config).await;
        info!("connection established");
        defmt::unwrap!(conn)
    }
}
