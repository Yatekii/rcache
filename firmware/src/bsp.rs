#![no_std]

pub use cortex_m;
pub use cortex_m_rt;
pub use embedded_hal;
pub use nrf52840_hal as hal;

/// Exports traits that are usually needed when using this crate
pub mod prelude {
    pub use nrf52840_hal::prelude::*;
}

use nrf52840_hal::{
    gpio::{p0, p1, Floating, Input, Level, Output, Pin, PullUp, PushPull},
    pac::{CorePeripherals, Peripherals, UARTE0},
    uarte::Uarte,
    Delay,
};

use embedded_hal::digital::v2::{InputPin, OutputPin};

/// Provides access to all features of the nRF52840 dongle
#[allow(non_snake_case)]
pub struct Board {
    /// The nRF52's pins which are not otherwise occupied on the nRF52840 dongle
    pub pins: Pins,

    /// The LEDs on the nRF52840 dongle
    pub leds: Leds,

    /// The buttons on the nRF52840 dongle
    pub buttons: Buttons,

    pub gps_uart: Uarte<UARTE0>,

    pub delay_source: Delay,

    pub lock: Lock,
}

impl Board {
    pub fn new(c: CorePeripherals, p: Peripherals) -> Self {
        let pins0 = p0::Parts::new(p.P0);
        let pins1 = p1::Parts::new(p.P1);

        let rxd = pins0.p0_15.into_floating_input().degrade();
        let txd = pins0.p0_17.into_push_pull_output(Level::Low).degrade();

        let mut delay_source = Delay::new(c.SYST);
        let pins = nrf52840_hal::uarte::Pins {
            rxd,
            txd,
            cts: None,
            rts: None,
        };

        let gps_uart = Uarte::new(
            p.UARTE0,
            pins,
            nrf52840_hal::uarte::Parity::EXCLUDED,
            nrf52840_hal::uarte::Baudrate::BAUD9600,
        );

        let lock = Lock::new(pins0.p0_31.degrade());

        Board {
            pins: Pins {
                p0_02: pins0.p0_02,
                p0_09: pins0.p0_09,
                p0_10: pins0.p0_10,
                p0_13: pins0.p0_13,
                p0_20: pins0.p0_20,
                p0_22: pins0.p0_22,
                p0_24: pins0.p0_24,
                p0_29: pins0.p0_29.into_push_pull_output(Level::Low),
                p1_10: pins1.p1_10,
                p1_13: pins1.p1_13,
                p1_15: pins1.p1_15,
            },

            leds: Leds {
                led1: Led::new(pins0.p0_06.degrade()),
                led2_r: Led::new(pins0.p0_08.degrade()),
                led2_g: Led::new(pins1.p1_09.degrade()),
                led2_b: Led::new(pins0.p0_12.degrade()),
            },

            buttons: Buttons {
                sw1: Button::new(pins1.p1_06.degrade()),
            },

            gps_uart,

            delay_source,

            lock,
        }
    }
}

/// The nRF52 pins that are available on the nRF52840DK
#[allow(non_snake_case)]
pub struct Pins {
    pub p0_02: p0::P0_02<Input<Floating>>,
    pub p0_09: p0::P0_09<Input<Floating>>,
    pub p0_10: p0::P0_10<Input<Floating>>,
    pub p0_13: p0::P0_13<Input<Floating>>,
    pub p0_20: p0::P0_20<Input<Floating>>,
    pub p0_22: p0::P0_22<Input<Floating>>,
    pub p0_24: p0::P0_24<Input<Floating>>,
    pub p0_29: p0::P0_29<Output<PushPull>>,
    pub p1_10: p1::P1_10<Input<Floating>>,
    pub p1_13: p1::P1_13<Input<Floating>>,
    pub p1_15: p1::P1_15<Input<Floating>>,
}

/// The LEDs on the nRF52840 dongle
pub struct Leds {
    /// nRF52840 dongle: LED1, nRF52: P0.06
    pub led1: Led,

    /// nRF52840 dongle: LED2R, nRF52: P0.08
    pub led2_r: Led,

    /// nRF52840 dongle: LED2G, nRF52: P1.09
    pub led2_g: Led,

    /// nRF52840 dongle: LED2B, nRF52: P0.12
    pub led2_b: Led,
}

/// An LED on the nRF52840 dongle
pub struct Led(Pin<Output<PushPull>>);

impl Led {
    fn new<Mode>(pin: Pin<Mode>) -> Self {
        Led(pin.into_push_pull_output(Level::High))
    }

    /// Enable the LED
    pub fn enable(&mut self) {
        self.0.set_low().unwrap()
    }

    /// Disable the LED
    pub fn disable(&mut self) {
        self.0.set_high().unwrap()
    }
}

/// The Buttons on the nRF52840 dongle
pub struct Buttons {
    /// nRF52840 dongle: Button 1, nRF52: P1.06
    pub sw1: Button,
}

/// A Button on the nRF52840 dongle
pub struct Button(Pin<Input<PullUp>>);

impl Button {
    fn new<Mode>(pin: Pin<Mode>) -> Self {
        Button(pin.into_pullup_input())
    }

    /// Button is pressed
    pub fn is_pressed(&self) -> bool {
        self.0.is_low().unwrap()
    }

    /// Button is released
    pub fn is_released(&self) -> bool {
        self.0.is_high().unwrap()
    }
}

pub enum Lock {
    Locked(Pin<Input<Floating>>),
    Open(Pin<Output<PushPull>>),
}

impl Lock {
    pub fn new<MODE>(pin: Pin<MODE>) -> Self {
        Lock::Locked(pin.into_floating_input())
    }

    pub fn open(&mut self) {
        unsafe {
            replace_with::replace_with_or_abort_unchecked(self, |self_| match self_ {
                Lock::Locked(pin) => Lock::Open(pin.into_push_pull_output(Level::Low)),
                Lock::Open(pin) => Lock::Open(pin),
            });
        }
    }

    pub fn lock(&mut self) {
        unsafe {
            replace_with::replace_with_or_abort_unchecked(self, |self_| match self_ {
                Lock::Open(pin) => Lock::Locked(pin.into_floating_input()),
                Lock::Locked(pin) => Lock::Locked(pin),
            });
        }
    }
}
