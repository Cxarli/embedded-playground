
#![deny(unsafe_code)]
#![no_std]
#![no_main]
#![feature(try_trait)]
#![allow(unused_imports, unused_mut, unused_variables)]

#[cfg(feature="semi")]
extern crate panic_semihosting;
#[cfg(feature="semi")]
use cortex_m_semihosting::hio;

#[cfg(not(feature="semi"))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

use core::convert::Infallible;
use core::fmt::{Debug, Write};
use cortex_m_rt::entry;
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
    #[cfg(feature="semi")]
    // connect stdout
    let mut stdout = hio::hstdout()?;

    // start timer
    let dev_peripherals = pac::Peripherals::take()?;
    let mut flash = dev_peripherals.FLASH.constrain();
    let mut radio_clock = dev_peripherals.RCC.constrain();
    let clocks = radio_clock.cfgr.freeze(&mut flash.acr);
    let tim2 = timer::Timer::tim2(dev_peripherals.TIM2, &clocks, &mut radio_clock.apb1);
    let tim3 = timer::Timer::tim3(dev_peripherals.TIM3, &clocks, &mut radio_clock.apb1);
    let mut main_countdown = tim2.start_count_down(400.ms());
    
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
    matrix.power_on()?;

    /*
    // speaker
    let mut pa6 = gpioa.pa6.into_alternate_push_pull(&mut gpioa.crl);
    let mut speaker = tim3.pwm(pa6, &mut afio.mapr, 300.ms());
    speaker.set_duty(pwm::Channel::C1, speaker.get_max_duty());
    speaker.enable(pwm::Channel::C1);
    */
    
    // initial matrix state
    let text = {
        use patterns::*;

        #[cfg(feature="semi")]
        let res = [merge(&s, &e), merge35(&e3, &m5), merge53(&m5, &i3), merge(&i, &blank), merge(&blank, &blank), merge(&blank, &s)];
        
        #[cfg(not(feature="semi"))]
        let res = [&C, &h, &a, &r, &l, &i, &e, &blank, &blank];
        // let res = [&h, &e, &l, &l, &o, &comma, &w, &o, &r, &l, &d, &excl, &blank, &blank];

        res
    };
    let mut i = 0;
    
    #[cfg(feature="semi")]
    write!(stdout, "Hello, world!\n")?;

    // main loop
    loop {

        #[cfg(feature="semi")]
        {
            matrix.write_raw(0, &text[i])?;
        }

        #[cfg(not(feature="semi"))]
        {
            let cur = text[i];
            let next = if i < text.len() - 1 { text[i + 1] } else { text[0] };
            matrix.write_raw(0, &patterns::merge(cur, next))?;
        }

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
