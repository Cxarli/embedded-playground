
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
// use max7219::MAX7219;
use nb::block;
use one_wire_bus::OneWire;
use stm32f1xx_hal::{/*delay::Delay,*/ pac, prelude::*, timer::Timer};

// mod numpad;
// use numpad::*;
// mod patterns;

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

/// Get the temperature probe connected on the given pin, if any
fn _get_temp_probe<T, U>(
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

struct Bitstring {
    val: u64,
    size: usize,
}

impl core::iter::FromIterator<bool> for Bitstring {
    fn from_iter<I: IntoIterator<Item=bool>>(i: I) -> Self {
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
    // let core_peripherals = cortex_m::Peripherals::take()?;
    let dev_peripherals = pac::Peripherals::take()?;
    let mut flash = dev_peripherals.FLASH.constrain();
    let mut radio_clock = dev_peripherals.RCC.constrain();
    let clocks = radio_clock.cfgr.freeze(&mut flash.acr);
    // let mut afio = dev_peripherals.AFIO.constrain(&mut radio_clock.apb2);
    // let mut gpioa = dev_peripherals.GPIOA.split(&mut radio_clock.apb2);
    let mut gpiob = dev_peripherals.GPIOB.split(&mut radio_clock.apb2);
    let mut gpioc = dev_peripherals.GPIOC.split(&mut radio_clock.apb2);
    let tim2 = Timer::tim2(dev_peripherals.TIM2, &clocks, &mut radio_clock.apb1);
    let mut main_countdown = tim2.start_count_down(100.ms());
    // let mut delay = Delay::new(core_peripherals.SYST, clocks);
    // let (pa15, pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);


    /*
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
        writeln!(stdout, "{:?}", pins.iter().map(|x| x.is_high().unwrap()).collect::<Bitstring>())?;
        block!(main_countdown.wait())?;
    }
    */


    // claim all GPIO pins
    let mut pc13 = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // numpad
    /*
    let pa15 = pa15.into_push_pull_output(&mut gpioa.crh);
    let pb3 = pb3.into_pull_down_input(&mut gpiob.crl);
    let pb4 = pb4.into_pull_down_input(&mut gpiob.crl);
    let pb5 = gpiob.pb5.into_pull_down_input(&mut gpiob.crl);
    */

    // temp probe
    // let pb12 = gpiob.pb12.into_open_drain_output(&mut gpiob.crh);
    // let mut temp_probe = get_temp_probe(pb12, &mut delay, &mut stdout)?;

    // matrix
    /*
    let pb8 = gpiob.pb8.into_push_pull_output(&mut gpiob.crh);
    let pb7 = gpiob.pb7.into_push_pull_output(&mut gpiob.crl);
    let pb6 = gpiob.pb6.into_push_pull_output(&mut gpiob.crl);
    */

    // get 4x4 numpad
    /* let mut numpad = {
        let row_0 = Some(pa15.downgrade());
        let col_0 = Some(pb3.downgrade());
        let col_1 = Some(pb4.downgrade());
        let col_2 = Some(pb5.downgrade());
        Numpad::new::<Error>([row_0, None, None, None], [col_0, col_1, col_2, None])?
    }; */

    // get LED matrix
    /* let mut matrix = MAX7219::from_pins(
        /*displays*/ 1, /*data*/ pb7, /*cs*/ pb8, /*sck*/ pb6,
    )?; */

    // initial matrix state
    // let mut pixels = patterns::Chess;

    // turn off the on-board led
    pc13.set_low()?;



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
        }
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

    /*
    // main loop
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

        // read the numpad
        let buttons = numpad.read::<Error>()?;
        writeln!(stdout, "buttons {:#06b}  ", 0x000f & buttons)?;

        // check which buttons are pressed
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

            // ignore other combinations
            _ => {}
        }

        // write the pixels to the matrix
        matrix.power_on()?;
        matrix.write_raw(0, &pixels)?;

        // wait before we loop
        block!(main_countdown.wait())?;
    }
    */
}

#[entry]
fn main() -> ! {
    _main().unwrap();
    panic!()
}
