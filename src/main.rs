
#![deny(unsafe_code)]
#![no_std]
#![no_main]
#![feature(try_trait)]
#![allow(unused_imports, unused_mut, unused_variables)]

extern crate panic_semihosting;

use core::convert::Infallible;
use core::fmt::{Debug, Write};
use cortex_m_rt::entry;
use cortex_m_semihosting::hio;
use max7219::MAX7219;
use nb::block;
use embedded_hal::digital::v2::OutputPin;
use stm32f1xx_hal::{pwm, pac, timer, prelude::*};

mod patterns;

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
    (Max7219, max7219::DataError),
    (Owb, one_wire_bus::OneWireError<Infallible>),
);


/// Wrapper around main which supports returning errors
fn _main() -> Result<(), Error> {
    // connect stdout
    let mut stdout = hio::hstdout()?;

    // start timer
    let dev_peripherals = pac::Peripherals::take()?;
    let mut flash = dev_peripherals.FLASH.constrain();
    let mut radio_clock = dev_peripherals.RCC.constrain();
    let clocks = radio_clock.cfgr.freeze(&mut flash.acr);
    let tim2 = timer::Timer::tim2(dev_peripherals.TIM2, &clocks, &mut radio_clock.apb1);
    let tim3 = timer::Timer::tim3(dev_peripherals.TIM3, &clocks, &mut radio_clock.apb1);
    let mut main_countdown = tim2.start_count_down(300.ms());
    
    // connect gpio
    let mut gpiob = dev_peripherals.GPIOB.split(&mut radio_clock.apb2);
    let mut afio = dev_peripherals.AFIO.constrain(&mut radio_clock.apb2);
    let mut gpioa = dev_peripherals.GPIOA.split(&mut radio_clock.apb2);
    let (_pa15, _pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);
    
    // build matrix
    let pb9 = gpiob.pb9.into_push_pull_output(&mut gpiob.crh); // green
    let pb8 = gpiob.pb8.into_push_pull_output(&mut gpiob.crh); // orange
    let pb4 = pb4.into_push_pull_output(&mut gpiob.crl); // yellow
    let mut matrix = MAX7219::from_pins(
        /*displays*/ 1,
        
        /* data = sda/sdi */   pb9,
        /* cs = chip select */ pb4,
        /* sck = clock */      pb8,
    )?;

    write!(stdout, "Preparing... ")?;
    for _ in 0..=9 {
        block!(main_countdown.wait())?;
    }
    write!(stdout, "Go!\n")?;

    matrix.power_on()?;
    block!(main_countdown.wait())?;

    // speaker
    let mut pa6 = gpioa.pa6.into_alternate_push_pull(&mut gpioa.crl);
    let mut speaker = tim3.pwm(pa6, &mut afio.mapr, 300.ms());

    // Start using the channels
    speaker.set_duty(pwm::Channel::C1, speaker.get_max_duty());
    speaker.enable(pwm::Channel::C1);
    
    // initial matrix state
    let text = {
        use patterns::*;
        [&h, &e, &l, &l, &o, &comma, &w, &o, &r, &l, &d, &excl, &blank, &blank]
    };
    let mut i = 0;
    
    // main loop
    loop {
        // write!(stdout, "hi\n")?;

        let cur = text[i];
        let next = if i < text.len() - 1 { text[i + 1] } else { text[0] };
        matrix.write_raw(0, &patterns::merge(cur, next))?;

        i = (i + 1) % text.len();

        // wait before we loop
        block!(main_countdown.wait())?;
    }
}

#[entry]
fn main() -> ! {
    _main().unwrap();
    panic!()
}
