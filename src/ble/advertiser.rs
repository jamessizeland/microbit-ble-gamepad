use defmt::info;
use trouble_host::prelude::*;

/// BLE advertiser
pub struct AdvertiserBuilder<'d, C: Controller> {
    /// Name of the device
    name: &'d str,
    peripheral: Peripheral<'d, C>,
}

pub struct Advertiser<'d, C: Controller> {
    advertiser_data: [u8; 31],
    scan_data: [u8; 4],
    peripheral: Peripheral<'d, C>,
}

/// A BLE advertiser
impl<'d, C: Controller> AdvertiserBuilder<'d, C> {
    /// Create a new advertiser builder
    pub fn new(name: &'d str, peripheral: Peripheral<'d, C>) -> Self {
        Self { name, peripheral }
    }
    /// Build the advertiser
    pub fn build(self) -> Result<Advertiser<'d, C>, Error> {
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

impl<'d, C: Controller> Advertiser<'d, C> {
    /// Advertise and connect to a device with the given name
    pub async fn advertise(&mut self) -> Result<Connection<'d>, BleHostError<C::Error>> {
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
