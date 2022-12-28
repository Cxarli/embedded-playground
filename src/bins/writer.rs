#![deny(unsafe_code)]
#![no_std]
#![no_main]
#![feature(try_trait)]

extern crate panic_semihosting;

use core::convert::Infallible;
use core::fmt::{Debug, Write};
use cortex_m_rt::entry;
use cortex_m_semihosting::hio;
use embedded_hal::digital::v2::OutputPin;
use nb::block;
use stm32f1xx_hal::{pac, prelude::*, timer::Timer};

// Combine all possible errors into one single Error

macro_rules! build_error {
    ( $(($x:ident, $y:ty)),* $(,)? ) => {

        #[derive(Debug)]
        enum Error {
            $(
                $x($y)
            ),*
        }

        $(
            impl From<$y> for Error {
                fn from(item: $y) -> Self {
                    Self::$x(item)
                }
            }
        )*
    }
}

build_error!(
    (NoneOption, core::option::NoneError),
    (Unit, ()),
    (Void, void::Void),
    (Fmt, core::fmt::Error),
    (Infallible, Infallible),
    (I2c, stm32f1xx_hal::i2c::Error),
);

fn _main() -> Result<(), Error> {
    // get access to all required peripherals
    let mut stdout = hio::hstdout()?;

    let dev_peripherals = pac::Peripherals::take()?;
    let mut flash = dev_peripherals.FLASH.constrain();
    let mut radio_clock = dev_peripherals.RCC.constrain();
    let clocks = radio_clock.cfgr.freeze(&mut flash.acr);
    let mut gpiob = dev_peripherals.GPIOB.split(&mut radio_clock.apb2);
    let tim2 = Timer::tim2(dev_peripherals.TIM2, &clocks, &mut radio_clock.apb1);
    let mut main_countdown = tim2.start_count_down(100.ms());

    let mut pb12 = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
    pb12.set_low()?;

    macro_rules! write_byte {
        ($byte: expr) => {
            let mut b = $byte;

            write!(stdout, "{} {:0x}\n", b as char, b)?;

            for _ in 1..=8 {
                if b & (1 << 7) != 0 {
                    pb12.set_high()?;
                } else {
                    pb12.set_low()?;
                }

                b <<= 1;

                block!(main_countdown.wait())?;
            }

            pb12.set_low()?;
            block!(main_countdown.wait())?;
        };
    }

    loop {
        write!(stdout, "Waiting...\n")?;
        pb12.set_low()?;
        for _ in 1..=32 {
            block!(main_countdown.wait())?;
        }

        write!(stdout, "Timing...\n")?;
        for _ in 1..=4 {
            pb12.set_high()?;
            block!(main_countdown.wait())?;
            pb12.set_low()?;
            block!(main_countdown.wait())?;
        }

        write!(stdout, "Writing...\n")?;
        for c in "Hello World!".bytes() {
            write_byte!(c);
        }
    }
}

#[entry]
fn main() -> ! {
    _main().unwrap();
    panic!()
}
