#![deny(unsafe_code)]
#![no_std]
#![no_main]
#![feature(try_trait)]

extern crate panic_semihosting;

use core::convert::Infallible;
use core::fmt::{Debug, Write};
use cortex_m_rt::entry;
use cortex_m_semihosting::hio;
use embedded_hal::digital::v2::InputPin;
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

struct Bitstring {
    val: u64,
    size: usize,
}

impl core::iter::FromIterator<bool> for Bitstring {
    fn from_iter<I: IntoIterator<Item = bool>>(i: I) -> Self {
        let mut val = 0;
        let mut size = 0;

        for x in i {
            val <<= 1;
            val |= x as u64;
            size += 1;
        }

        Self { val, size }
    }
}

impl Debug for Bitstring {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::fmt::Result {
        fmt.write_fmt(format_args!("{:b}", 1 << self.size | self.val))
    }
}

/// Wrapper around main which supports returning errors
fn _main() -> Result<(), Error> {
    // get access to all required peripherals
    let mut stdout = hio::hstdout()?;

    let dev_peripherals = pac::Peripherals::take()?;
    let mut flash = dev_peripherals.FLASH.constrain();
    let mut radio_clock = dev_peripherals.RCC.constrain();
    let clocks = radio_clock.cfgr.freeze(&mut flash.acr);
    let mut afio = dev_peripherals.AFIO.constrain(&mut radio_clock.apb2);
    let mut gpioa = dev_peripherals.GPIOA.split(&mut radio_clock.apb2);
    let mut gpiob = dev_peripherals.GPIOB.split(&mut radio_clock.apb2);
    let mut gpioc = dev_peripherals.GPIOC.split(&mut radio_clock.apb2);
    let tim2 = Timer::tim2(dev_peripherals.TIM2, &clocks, &mut radio_clock.apb1);
    let mut main_countdown = tim2.start_count_down(100.ms());

    let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

    let pins = [
        gpiob.pb11.into_pull_down_input(&mut gpiob.crh).downgrade(),
        gpiob.pb10.into_pull_down_input(&mut gpiob.crh).downgrade(),
        gpiob.pb1.into_pull_down_input(&mut gpiob.crl).downgrade(),
        gpiob.pb0.into_pull_down_input(&mut gpiob.crl).downgrade(),
        gpioa.pa7.into_pull_down_input(&mut gpioa.crl).downgrade(),
        gpioa.pa6.into_pull_down_input(&mut gpioa.crl).downgrade(),
        gpioa.pa5.into_pull_down_input(&mut gpioa.crl).downgrade(),
        gpioa.pa4.into_pull_down_input(&mut gpioa.crl).downgrade(),
        gpioa.pa3.into_pull_down_input(&mut gpioa.crl).downgrade(),
        gpioa.pa2.into_pull_down_input(&mut gpioa.crl).downgrade(),
        gpioa.pa1.into_pull_down_input(&mut gpioa.crl).downgrade(),
        gpioa.pa0.into_pull_down_input(&mut gpioa.crl).downgrade(),
        gpioc.pc15.into_pull_down_input(&mut gpioc.crh).downgrade(),
        gpioc.pc14.into_pull_down_input(&mut gpioc.crh).downgrade(),
        // gpioc.pc13.into_pull_down_input(&mut gpioc.crh).downgrade(),
        gpiob.pb12.into_pull_down_input(&mut gpiob.crh).downgrade(),
        gpiob.pb13.into_pull_down_input(&mut gpiob.crh).downgrade(),
        gpiob.pb14.into_pull_down_input(&mut gpiob.crh).downgrade(),
        gpiob.pb15.into_pull_down_input(&mut gpiob.crh).downgrade(),
        gpioa.pa8.into_pull_down_input(&mut gpioa.crh).downgrade(),
        gpioa.pa9.into_pull_down_input(&mut gpioa.crh).downgrade(),
        gpioa.pa10.into_pull_down_input(&mut gpioa.crh).downgrade(),
        gpioa.pa11.into_pull_down_input(&mut gpioa.crh).downgrade(),
        gpioa.pa12.into_pull_down_input(&mut gpioa.crh).downgrade(),
        pa15.into_pull_down_input(&mut gpioa.crh).downgrade(),
        pb3.into_pull_down_input(&mut gpiob.crl).downgrade(),
        pb4.into_pull_down_input(&mut gpiob.crl).downgrade(),
        gpiob.pb5.into_pull_down_input(&mut gpiob.crl).downgrade(),
        gpiob.pb6.into_pull_down_input(&mut gpiob.crl).downgrade(),
        gpiob.pb7.into_pull_down_input(&mut gpiob.crl).downgrade(),
        gpiob.pb8.into_pull_down_input(&mut gpiob.crh).downgrade(),
        gpiob.pb9.into_pull_down_input(&mut gpiob.crh).downgrade(),
    ];

    loop {
        writeln!(
            stdout,
            "{:?}",
            pins.iter()
                .map(|x| x.is_high().unwrap())
                .collect::<Bitstring>()
        )?;
        block!(main_countdown.wait())?;
    }
}

#[entry]
fn main() -> ! {
    _main().unwrap();
    panic!()
}
