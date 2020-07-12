#![no_std]
#![no_main]

mod bsp;

pub use cortex_m;
pub use cortex_m_rt;
pub use embedded_hal;
use nmea0183 as nmea;
pub use nrf52840_hal as hal;
use rtic::app;

//use panic_halt as _;
use nrf52840_hal::uarte::Uarte;
use panic_rtt_target as _;
use rtt_target::rprintln;
use rtt_target::rtt_init_print;

#[app(device = crate::hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        UART: Uarte<hal::pac::UARTE0>,
        RX_BUFFER: [u8; 20],
        NMEA: nmea::Parser,
        BUTTON: bsp::Button,
        LOCK: bsp::Lock,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        rtt_init_print!();
        rprintln!("init");

        let c = cx.core;
        let p = cx.device;
        let mut bsp = bsp::Board::new(c, p);

        bsp.lock.open();

        let resources = init::LateResources {
            UART: bsp.gps_uart,
            RX_BUFFER: [0; 20],
            NMEA: nmea::Parser::new(),
            BUTTON: bsp.buttons.sw1,
            LOCK: bsp.lock,
        };

        resources
    }

    #[idle(resources=[UART, RX_BUFFER, NMEA, BUTTON, LOCK])]
    fn idle(cx: idle::Context) -> ! {
        rprintln!("idle");

        loop {
            if cx.resources.BUTTON.is_pressed() {
                cx.resources.LOCK.open();
            } else {
                cx.resources.LOCK.lock();
            }

            match cx.resources.UART.read(cx.resources.RX_BUFFER) {
                Ok(_) => {
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
                // If we encounter an error on the UART, we just ignore it for now.
                Err(_) => (),
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
