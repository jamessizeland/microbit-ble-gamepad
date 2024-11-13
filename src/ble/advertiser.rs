use defmt::info;
use microbit_bsp::ble::SoftdeviceError;
use trouble_host::prelude::*;

use super::SdcPeripheral;

/// BLE advertiser
pub struct AdvertiserBuilder<'d> {
    /// Name of the device
    name: &'d str,
    peripheral: SdcPeripheral<'d>,
}

pub struct Advertiser<'d> {
    advertiser_data: [u8; 31],
    scan_data: [u8; 4],
    peripheral: SdcPeripheral<'d>,
}

/// A BLE advertiser
impl<'d> AdvertiserBuilder<'d> {
    /// Create a new advertiser builder
    pub fn new(name: &'d str, peripheral: SdcPeripheral<'d>) -> Self {
        Self { name, peripheral }
    }
    /// Build the advertiser
    pub fn build(self) -> Result<Advertiser<'d>, Error> {
        let name: &str;
        if self.name.len() > 22 {
            name = &self.name[..22];
            info!("Name truncated to {}", name);
        } else {
            name = self.name;
        }
        let mut advertiser_data = [0; 31];
        AdStructure::encode_slice(
            &[
                AdStructure::Flags(LE_GENERAL_DISCOVERABLE | BR_EDR_NOT_SUPPORTED),
                AdStructure::ServiceUuids16(&[Uuid::Uuid16([0x0f, 0x18])]),
                AdStructure::CompleteLocalName(name.as_bytes()),
            ],
            &mut advertiser_data[..],
        )?;
        #[rustfmt::skip]
        let scan_data: [u8;4] = [0; 4];
        Ok(Advertiser {
            advertiser_data,
            scan_data,
            peripheral: self.peripheral,
        })
    }
}

impl<'d> Advertiser<'d> {
    /// Advertise and connect to a device with the given name
    pub async fn advertise(&mut self) -> Result<Connection, BleHostError<SoftdeviceError>> {
        let mut advertiser = self
            .peripheral
            .advertise(
                &Default::default(),
                Advertisement::ConnectableScannableUndirected {
                    adv_data: &self.advertiser_data[..],
                    scan_data: &self.scan_data[..],
                },
            )
            .await?;
        info!("advertising");
        let conn = advertiser.accept().await?;
        info!("connection established");
        Ok(conn)
    }
}
