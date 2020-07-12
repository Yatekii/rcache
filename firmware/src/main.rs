#![no_std]
#![no_main]

pub use cortex_m;
pub use cortex_m_rt;
pub use embedded_hal;
use nmea0183 as nmea;
pub use nrf52840_hal as hal;
use rtic::app;

//use panic_halt as _;
use panic_rtt_target as _;
use rtt_target::rprint;
use rtt_target::rprintln;
use rtt_target::rtt_init_print;

use nrf52840_hal::delay::Delay;
use nrf52840_hal::gpio::*;
use nrf52840_hal::prelude::*;
use nrf52840_hal::uarte::Uarte;

// #[cortex_m_rt::entry]
// fn main() -> ! {
//     rtt_init_print!();

//     if let (Some(p), Some(c)) = (
//         nrf52840_hal::pac::Peripherals::take(),
//         nrf52840_hal::pac::CorePeripherals::take(),
//     ) {
//         let pins0 = p0::Parts::new(p.P0);
//         let mut led = pins0.p0_06.into_push_pull_output(Level::Low);

//         let rxd = pins0.p0_15.into_floating_input().degrade();
//         let txd = pins0.p0_17.into_push_pull_output(Level::Low).degrade();

//         let mut delay_source = Delay::new(c.SYST);
//         let pins = nrf52840_hal::uarte::Pins {
//             rxd,
//             txd,
//             cts: None,
//             rts: None,
//         };

//         let uart = Uarte::new(
//             p.UARTE0,
//             pins,
//             nrf52840_hal::uarte::Parity::EXCLUDED,
//             nrf52840_hal::uarte::Baudrate::BAUD9600,
//         );

//         // struct Uart {
//         //     uart: Uarte<hal::target::UARTE0>,
//         // }

//         // impl embedded_hal::serial::Read<u8> for Uart {
//         //     type Error = nrf52840_hal::uarte::Error;

//         //     fn read(&mut self) -> nb::Result<u8, Self::Error> {
//         //         let mut buffer = [0u8; 1];
//         //         self.uart.read(&mut buffer)?;
//         //         Ok(buffer[0])
//         //     }
//         // }

//         // let mut uart = Uart { uart };

//         delay_source.delay_ms(1u8);

//         // let mut driver = ublox::new_serial_driver(Uart { uart });
//         // driver.setup(&mut delay_source).unwrap();

//         rprintln!("System booted!");

//         // loop {
//         //     let b = uart.read().unwrap();
//         //     rprint!("{}", b as char);
//         //     delay_source.delay_ms(1u8);
//         // }
//     }

//     loop {}
// }

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        UART: Uarte<hal::target::UARTE0>,
        RX_BUFFER: [u8; 20],
        NMEA: nmea::Parser,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        rprintln!("init");

        let c = cx.core;
        let p = cx.device;
        let pins0 = p0::Parts::new(p.P0);
        let mut led = pins0.p0_06.into_push_pull_output(Level::Low);

        let rxd = pins0.p0_15.into_floating_input().degrade();
        let txd = pins0.p0_17.into_push_pull_output(Level::Low).degrade();

        let mut delay_source = Delay::new(c.SYST);
        let pins = nrf52840_hal::uarte::Pins {
            rxd,
            txd,
            cts: None,
            rts: None,
        };

        let uart = Uarte::new(
            p.UARTE0,
            pins,
            nrf52840_hal::uarte::Parity::EXCLUDED,
            nrf52840_hal::uarte::Baudrate::BAUD9600,
        );

        let mut resources = init::LateResources {
            UART: uart,
            RX_BUFFER: [0; 20],
            NMEA: nmea::Parser::new(),
        };

        resources
    }

    #[idle(resources=[UART, RX_BUFFER, NMEA])]
    fn idle(cx: idle::Context) -> ! {
        rprintln!("idle");

        loop {
            cx.resources.UART.read(cx.resources.RX_BUFFER);
            // for c in &cx.resources.RX_BUFFER[..] {
            //     rprint!("{}", *c as char);
            // }

            for b in &cx.resources.RX_BUFFER[..] {
                if let Some(result) = cx.resources.NMEA.parse_from_byte(*b) {
                    match result {
                        Ok(nmea::ParseResult::GGA(Some(gga))) => rprintln!("{:?}", gga), // Got GGA sentence
                        Ok(nmea::ParseResult::GGA(None)) => rprintln!("No GGA."), // Got GGA sentence without valid data, receiver ok but has no solution
                        Ok(s) => rprintln!("{:?}", s), // Some other sentences..
                        Err(e) => rprintln!("{:?}", e), // Got parse error
                    }
                }
            }
        }
    }

    // #[task(binds=UARTE0_UART0, resources=[UART, RX_BUFFER])]
    // fn UART(cx: UART::Context) {
    //     for c in &cx.resources.RX_BUFFER[..] {
    //         rprint!("{}", *c as char);
    //     }
    //     cx.resources.UART.read(cx.resources.RX_BUFFER);
    // }
};
