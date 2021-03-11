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
use max7219::MAX7219;
use nb::block;
use one_wire_bus::OneWire;
use stm32f1xx_hal::{delay::Delay, pac, prelude::*, timer::Timer};

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
    (NoneError, core::option::NoneError),
    (Unit, ()),
    (Void, void::Void),
    (FmtError, core::fmt::Error),
    (Infallible, Infallible),
    (I2c, stm32f1xx_hal::i2c::Error),
    (Max7219, max7219::DataError),
    (Owb, one_wire_bus::OneWireError<Infallible>),
);

mod numpad;
use numpad::*;
mod patterns;

fn get_temp_probe<T, U>(
    pin: T,
    delay: &mut U,
    stdout: &mut hio::HStdout,
) -> Result<Option<(Ds18b20, OneWire<T>)>, Error>
where
    T: InputPin<Error = Infallible> + OutputPin<Error = Infallible>,
    U: DelayMs<u16> + DelayUs<u16>,
{
    let mut owb = OneWire::new(pin)?;

    let mut devs = owb.devices(false, delay);
    let probe: Option<Ds18b20> = loop {
        match devs.next() {
            Some(Ok(addr)) => {
                writeln!(stdout, "addr: {:?}", addr)?;
                match Ds18b20::new::<()>(addr) {
                    Ok(x) => break Some(x),
                    Err(e) => {
                        writeln!(stdout, "Ds::new     {:?}", e)?;
                    }
                }
            }
            Some(Err(e)) => {
                writeln!(stdout, "devs.next   {:?}", e)?;
            }
            None => {
                writeln!(stdout, "out of devices")?;
                break None;
            }
        }
    };

    if let Some(probe) = probe {
        probe.start_temp_measurement(&mut owb, delay)?;
        ds18b20::Resolution::Bits12.delay_for_measurement_time(delay);
        Ok(Some((probe, owb)))
    } else {
        owb.release_bus()?;
        Ok(None)
    }
}

fn _main() -> Result<(), Error> {
    let mut stdout = hio::hstdout()?;
    let core_peripherals = cortex_m::Peripherals::take()?;
    let dev_peripherals = pac::Peripherals::take()?;
    let mut flash = dev_peripherals.FLASH.constrain();
    let mut radio_clock = dev_peripherals.RCC.constrain();
    let clocks = radio_clock.cfgr.freeze(&mut flash.acr);
    let mut afio = dev_peripherals.AFIO.constrain(&mut radio_clock.apb2);
    let mut gpioa = dev_peripherals.GPIOA.split(&mut radio_clock.apb2);
    let mut gpiob = dev_peripherals.GPIOB.split(&mut radio_clock.apb2);
    let mut gpioc = dev_peripherals.GPIOC.split(&mut radio_clock.apb2);
    let tim1 = Timer::tim1(dev_peripherals.TIM1, &clocks, &mut radio_clock.apb2);
    let tim2 = Timer::tim2(dev_peripherals.TIM2, &clocks, &mut radio_clock.apb1);
    let mut delay = Delay::new(core_peripherals.SYST, clocks);
    let (pa15, _pb3, _pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);
    let pa15 = pa15.into_open_drain_output(&mut gpioa.crh);
    let mut pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);
    let pb12 = gpiob.pb12.into_push_pull_output(&mut gpiob.crh);
    let pb13 = gpiob.pb13.into_pull_down_input(&mut gpiob.crh);
    let pb14 = gpiob.pb14.into_pull_down_input(&mut gpiob.crh);
    let pb15 = gpiob.pb15.into_pull_down_input(&mut gpiob.crh);
    let pb8 = gpiob.pb8.into_push_pull_output(&mut gpiob.crh);
    let pb7 = gpiob.pb7.into_push_pull_output(&mut gpiob.crl);
    let pb6 = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);
    let mut temp_probe = get_temp_probe(pa15, &mut delay, &mut stdout)?;


    // get 4x4 numpad
    let mut numpad = {
        let row_0 = Some(pb12.downgrade());
        let col_0 = Some(pb13.downgrade());
        let col_1 = Some(pb14.downgrade());
        let col_2 = Some(pb15.downgrade());
        Numpad::new(row_0, None, None, None, col_0, col_1, col_2, None)
    };

    let mut matrix = MAX7219::from_pins(
        /*displays*/ 1, /*data*/ pb7, /*cs*/ pb8, /*sck*/ pb6,
    )?;

    let mut pixels = patterns::Chess;

    let mut button_countdown = tim1.start_count_down(500.ms());
    let mut main_countdown = tim2.start_count_down(1.hz());

    pc13.set_high()?;

    loop {
        if let Some((ref probe, ref mut owb)) = temp_probe {
            let temp_sensor = probe.read_data(owb, &mut delay)?;
            write!(
                stdout,
                "temp {} mC",
                (temp_sensor.temperature * 1000.) as i32
            )?;
        }

        let buttons = numpad.read(&mut button_countdown);
        write!(stdout, "buttons {:#06b}  ", buttons)?;

        matrix.power_on()?;
        matrix.write_raw(0, &pixels)?;

        let one = buttons & Buttons::One != 0;
        let two = buttons & Buttons::Two != 0;
        let three = buttons & Buttons::Three != 0;

        match (one, two, three) {
            // holding 1
            (true, false, false) => {
                writeln!(stdout, "hold 1")?;
                pixels = patterns::One;
            }

            // holding 2
            (false, true, false) => {
                writeln!(stdout, "hold 2")?;
                pixels = patterns::Chess;
            }

            _ => {}
        }

        block!(main_countdown.wait())?;
    }
}

#[entry]
fn main() -> ! {
    _main().unwrap();
    panic!()
}
