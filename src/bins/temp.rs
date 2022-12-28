#![deny(unsafe_code)]
#![no_std]
#![no_main]
#![feature(try_trait)]

extern crate panic_semihosting;

use core::convert::Infallible;
use core::fmt::{Debug, Write};
use cortex_m_rt::entry;
use cortex_m_semihosting::hio;
use ds18b20::Ds18b20;
use embedded_hal::blocking::delay::{DelayMs, DelayUs};
use embedded_hal::digital::v2::{InputPin, OutputPin};
use nb::block;
use one_wire_bus::OneWire;
use stm32f1xx_hal::{delay::Delay, pac, prelude::*, timer::Timer};

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
    (Owb, one_wire_bus::OneWireError<Infallible>),
);

/// Get the temperature probe connected on the given pin, if any
fn get_temp_probe<T, U>(
    pin: T,
    delay: &mut U,
    stdout: &mut hio::HStdout,
) -> Result<Option<(Ds18b20, OneWire<T>)>, Error>
where
    T: InputPin<Error = Infallible> + OutputPin<Error = Infallible>,
    U: DelayMs<u16> + DelayUs<u16>,
{
    // initialise the OneWireBus
    let mut owb = OneWire::new(pin)?;

    // find the device
    let mut devs = owb.devices(false, delay);
    let probe: Option<Ds18b20> = loop {
        match devs.next() {
            // found a device on the bus
            Some(Ok(addr)) => {
                writeln!(stdout, "addr: {:?}", addr)?;

                // check if it's a temperature probe
                match Ds18b20::new::<()>(addr) {
                    Ok(x) => break Some(x),

                    Err(e) => writeln!(stdout, "Ds::new     {:?}", e)?,
                }
            }

            // found a device but it errored
            Some(Err(e)) => {
                writeln!(stdout, "devs.next   {:?}", e)?;
            }

            // no more devices
            None => {
                writeln!(stdout, "out of devices")?;
                break None;
            }
        }
    };

    // if we found a probe
    if let Some(probe) = probe {
        // start measurement
        probe.start_temp_measurement(&mut owb, delay)?;
        ds18b20::Resolution::Bits12.delay_for_measurement_time(delay);

        Ok(Some((probe, owb)))
    } else {
        // release the bus again
        owb.release_bus()?;

        Ok(None)
    }
}

/// Wrapper around main which supports returning errors
fn _main() -> Result<(), Error> {
    // get access to all required peripherals
    let mut stdout = hio::hstdout()?;
    let core_peripherals = cortex_m::Peripherals::take()?;
    let dev_peripherals = pac::Peripherals::take()?;
    let mut flash = dev_peripherals.FLASH.constrain();
    let mut radio_clock = dev_peripherals.RCC.constrain();
    let clocks = radio_clock.cfgr.freeze(&mut flash.acr);
    let mut gpiob = dev_peripherals.GPIOB.split(&mut radio_clock.apb2);
    let tim2 = Timer::tim2(dev_peripherals.TIM2, &clocks, &mut radio_clock.apb1);
    let mut main_countdown = tim2.start_count_down(100.ms());
    let mut delay = Delay::new(core_peripherals.SYST, clocks);

    // temp probe
    let pb12 = gpiob.pb12.into_open_drain_output(&mut gpiob.crh);
    let mut temp_probe = get_temp_probe(pb12, &mut delay, &mut stdout)?;

    loop {
        // read the temperature sensor
        if let Some((ref probe, ref mut owb)) = temp_probe {
            let temp_sensor = probe.read_data(owb, &mut delay)?;
            write!(
                stdout,
                "temp {} mC",
                (temp_sensor.temperature * 1000.) as i32
            )?;
        }

        block!(main_countdown.wait())?;
    }
}

#[entry]
fn main() -> ! {
    _main().unwrap();
    panic!()
}
