use crate::ble::build_sdc;
use crate::ble::{advertiser, mpsl_task};

use super::advertiser::Advertiser;
use super::hid::*;
use super::stick::*;
use defmt::info;
use defmt::unwrap;
use embassy_executor::Spawner;
use microbit_bsp::embassy_nrf::{bind_interrupts, peripherals::RNG, rng};
use nrf_sdc::mpsl::MultiprotocolServiceLayer;
use nrf_sdc::{self as sdc, mpsl, SoftdeviceController};
use static_cell::StaticCell;
use trouble_host::{prelude::*, Advertiser};

/// Size of L2CAP packets (ATT MTU is this - 4)
const L2CAP_MTU: usize = 251;

/// Max number of connections
const CONNECTIONS_MAX: usize = 1;

/// Max number of L2CAP channels.
const L2CAP_CHANNELS_MAX: usize = 2; // Signal + att

const MAX_ATTRIBUTES: usize = 10;

type Resources<C> = HostResources<C, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX, L2CAP_MTU>;

bind_interrupts!(struct Irq {
    RNG => rng::InterruptHandler<RNG>;
    SWI0_EGU0 => nrf_sdc::mpsl::LowPrioInterruptHandler;
    POWER_CLOCK => nrf_sdc::mpsl::ClockInterruptHandler;
    RADIO => nrf_sdc::mpsl::HighPrioInterruptHandler;
    TIMER0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
    RTC0 => nrf_sdc::mpsl::HighPrioInterruptHandler;
});

#[gatt_server]
pub struct GamepadServer {
    // pub bas: BatteryService,
    pub hid: ButtonService,
    pub stick: StickService,
}

impl GamepadServer<'_, '_, SoftdeviceController<'_>> {
    pub fn start_gatt(
        name: &'static str,
        spawner: Spawner,
        mpsl_peripherals: nrf_mpsl::Peripherals<'_>,
        sdc_peripherals: nrf_sdc::Peripherals<'_>,
        rng: RNG,
    ) -> Self {
        // ) -> (Self, Advertiser) {
        let lfclk_cfg = mpsl::raw::mpsl_clock_lfclk_cfg_t {
            source: mpsl::raw::MPSL_CLOCK_LF_SRC_RC as u8,
            rc_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_CTIV as u8,
            rc_temp_ctiv: mpsl::raw::MPSL_RECOMMENDED_RC_TEMP_CTIV as u8,
            accuracy_ppm: mpsl::raw::MPSL_DEFAULT_CLOCK_ACCURACY_PPM as u16,
            skip_wait_lfclk_started: mpsl::raw::MPSL_DEFAULT_SKIP_WAIT_LFCLK_STARTED != 0,
        };
        static MPSL: StaticCell<MultiprotocolServiceLayer> = StaticCell::new();
        let mpsl = MPSL.init(unwrap!(mpsl::MultiprotocolServiceLayer::new(
            mpsl_peripherals,
            Irq,
            lfclk_cfg
        )));
        spawner.must_spawn(mpsl_task(mpsl));

        let mut rng = rng::Rng::new(rng, Irq);

        let mut sdc_mem = sdc::Mem::<3312>::new();
        let controller = unwrap!(build_sdc(sdc_peripherals, &mut rng, mpsl, &mut sdc_mem));

        // Create a BLE GATT server and make it static
        let address = Address::random([0x41, 0x5A, 0xE3, 0x1E, 0x83, 0xE7]);
        info!("Our address = {:?}", address);

        let mut resources = Resources::new(PacketQos::None);
        let (stack, peripheral, _, runner) = trouble_host::new(controller, &mut resources)
            .set_random_address(address)
            .build();

        let server = GamepadServer::new_with_config(
            stack,
            GapConfig::Peripheral(PeripheralConfig {
                name: "TrouBLE",
                appearance: &appearance::GENERIC_POWER,
            }),
        );
        info!("Starting Gatt Server");
        // let advertiser = advertiser::AdvertiserBuilder::new(name, sd).build();
        // (server, advertiser)
        server
    }
}

/// A BLE GATT server
pub async fn gatt_server_task(
    server: &GamepadServer<'_, '_, SoftdeviceController<'_>>,
    conn: &Connection<'_>,
) {
    // gatt_server::run(&conn, server, |e| match e {
    //     // GamepadServerEvent::Bas(e) => match e {
    //     //     BatteryServiceEvent::BatteryLevelCccdWrite { notifications } => {
    //     //         defmt::info!("battery notifications: {}", notifications)
    //     //     }
    //     // },
    //     GamepadServerEvent::Hid(e) => match e {
    //         ButtonServiceEvent::ButtonACccdWrite { notifications }
    //         | ButtonServiceEvent::ButtonBCccdWrite { notifications }
    //         | ButtonServiceEvent::ButtonCCccdWrite { notifications }
    //         | ButtonServiceEvent::ButtonDCccdWrite { notifications }
    //         | ButtonServiceEvent::ButtonECccdWrite { notifications }
    //         | ButtonServiceEvent::ButtonFCccdWrite { notifications } => {
    //             info!("button: {}", notifications)
    //         }
    //     },
    //     GamepadServerEvent::Stick(e) => match e {
    //         StickServiceEvent::XCccdWrite { notifications }
    //         | StickServiceEvent::YCccdWrite { notifications } => {
    //             info!("stick: {}", notifications);
    //         }
    //     },
    // })
    // .await;
    todo!();
    info!("Gatt server task finished");
}
