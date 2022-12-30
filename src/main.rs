#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![feature(generic_arg_infer)]
#![no_std]
#![no_main]
#![allow(unused_imports, unused_mut, unused_variables)]

#[cfg(feature = "semi")]
extern crate panic_semihosting;
#[cfg(feature = "semi")]
use cortex_m_semihosting::hio;

#[cfg(not(feature = "semi"))]
#[panic_handler]
fn panic_handler(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

use core::convert::Infallible;
use core::fmt::{Debug, Write};
use cortex_m_rt::entry;
use embedded_hal::digital::v2::OutputPin;
use max7219::MAX7219;
use nb::block;
use stm32f1xx_hal::{pac, prelude::*, i2c, pwm, timer};
use lcd_1602_i2c::{self, Lcd};

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
    (Unit, ()),
    (Str, &'static str),
    (Void, void::Void),
    (Fmt, core::fmt::Error),
    (Infallible, Infallible),
    (I2c, stm32f1xx_hal::i2c::Error),
    (Max7219, max7219::DataError),
    (Owb, one_wire_bus::OneWireError<Infallible>),
    (Nb, nb::Error<stm32f1xx_hal::i2c::Error>),
);

/// Wrapper around main which supports returning errors
fn _main() -> Result<(), Error> {
    #[cfg(feature = "semi")]
    // connect stdout
    let mut stdout = hio::hstdout()?;
    #[cfg(feature = "semi")]
    write!(stdout, "Hello, world!\n")?;

    // get system handles
    let dev_peripherals = pac::Peripherals::take().ok_or(())?;
    let mut core_peripherals = cortex_m::Peripherals::take().unwrap();
    let mut flash = dev_peripherals.FLASH.constrain();
    let mut radio_clock = dev_peripherals.RCC.constrain();
    
    core_peripherals.DCB.enable_trace();
    core_peripherals.DWT.enable_cycle_counter();
    
    let clocks = radio_clock.cfgr.freeze(&mut flash.acr);
    let mut delay = stm32f1xx_hal::delay::Delay::new(core_peripherals.SYST, clocks);
    let tim2 = timer::Timer::tim2(dev_peripherals.TIM2, &clocks, &mut radio_clock.apb1);
    let tim3 = timer::Timer::tim3(dev_peripherals.TIM3, &clocks, &mut radio_clock.apb1);

    // start timer
    let mut main_countdown = tim2.start_count_down(150.ms());

    // connect gpio
    let mut gpiob = dev_peripherals.GPIOB.split(&mut radio_clock.apb2);
    let mut afio = dev_peripherals.AFIO.constrain(&mut radio_clock.apb2);
    let mut gpioa = dev_peripherals.GPIOA.split(&mut radio_clock.apb2);
    let (_pa15, _pb3, pb4) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);

    // build LCD over I2C
    let pb11 = gpiob.pb11.into_alternate_open_drain(&mut gpiob.crh); // green
    let pb10 = gpiob.pb10.into_alternate_open_drain(&mut gpiob.crh); // orange
    
    let mut bus = i2c::BlockingI2c::i2c2(
        dev_peripherals.I2C2,
        (/* sck = clock */ pb10, /* data = sda/sdi */ pb11),
        i2c::Mode::Standard {
            frequency: 200_000.hz(),
        },
        clocks,
        &mut radio_clock.apb1,
        /* start_timeout_us */
        1000,
        /* start_retries */
        10,
        /* addr_timeout_us */
        1000,
        /* data_timeout_us */
        1000,
    );

    // let mut lcd = Lcd::new(bus, 0x27, 0x20, &mut delay)?;
    // lcd.set_cursor(lcd_1602_i2c::Cursor::On)?;
    // lcd.write_str("Hello world!")?;


    let addr = 0x27;

    for _ in 0..=3 {
        // Function set: 8-bit, 2-line, 5x8 pixels
        #[cfg(feature = "semi")]
        write!(stdout, "function set\n")?;
        bus.write(addr, &[0b_00_1_110_00])?;
        delay.delay_ms(100u16);
    }

    #[cfg(feature = "semi")]
    write!(stdout, "busy flag\n")?;
    let mut buffer: [u8; 1] = [0u8];
    bus.write_read(addr, &[], &mut buffer)?;
    #[cfg(feature = "semi")]
    write!(stdout, "ret: {:08b}\n", buffer[0])?;
    
    // Display On
    #[cfg(feature = "semi")]
    write!(stdout, "display on\n")?;
    bus.write(addr, &[0b_00001_100])?;
    delay.delay_ms(100u16);

    // Clear display
    #[cfg(feature = "semi")]
    write!(stdout, "clear display\n")?;
    bus.write(addr, &[0b_0000000_1])?;
    delay.delay_ms(100u16);

    // display on
    #[cfg(feature = "semi")]
    write!(stdout, "display on\n")?;
    bus.write(addr, &[0b_00001_100])?;
    delay.delay_ms(100u16);

    // Entry Mode
    #[cfg(feature = "semi")]
    write!(stdout, "entry mode\n")?;
    bus.write(addr, &[0b_000001_11])?;
    delay.delay_ms(100u16);

    // display on
    #[cfg(feature = "semi")]
    write!(stdout, "display on\n")?;
    bus.write(addr, &[0b_00001_100])?;
    delay.delay_ms(100u16);

    // #[cfg(feature = "semi")]
    // write!(stdout, "set ddram 11\n")?;
    // bus.write(addr, &[0b_1_1000010])?;
    // delay.delay_ms(100u16);

    // #[cfg(feature = "semi")]
    // write!(stdout, "set ddram 10\n")?;
    // bus.write(addr, &[0b_1_1000000])?;
    // delay.delay_ms(100u16);

    // #[cfg(feature = "semi")]
    // write!(stdout, "set ddram 01\n")?;
    // bus.write(addr, &[0b_1_0000010])?;
    // delay.delay_ms(100u16);

    #[cfg(feature = "semi")]
    write!(stdout, "write character\n")?;
    // Write character 'f'
    bus.write(addr, &[0b_01100110])?;
    delay.delay_ms(100u16);

    // display on
    #[cfg(feature = "semi")]
    write!(stdout, "display on\n")?;
    bus.write(addr, &[0b_00001_100])?;
    delay.delay_ms(100u16);

    // build matrix
    let pb9 = gpiob.pb9.into_push_pull_output(&mut gpiob.crh); // green
    let pb8 = gpiob.pb8.into_push_pull_output(&mut gpiob.crh); // orange
    let pb4 = pb4.into_push_pull_output(&mut gpiob.crl); // yellow
    let mut matrix = MAX7219::from_pins(
        /*displays*/ 1, /* data = sda/sdi */ pb9, /* cs = chip select */ pb4,
        /* sck = clock */ pb8,
    )?;
    matrix.power_on()?;
    matrix_fun(&mut matrix, &mut main_countdown)
}


fn matrix_fun<T: max7219::connectors::Connector>(
    matrix: &mut MAX7219<T>,
    main_countdown: &mut stm32f1xx_hal::timer::CountDownTimer<stm32f1xx_hal::pac::TIM2>,
) -> Result<(), Error> {
    struct Text<const N: usize> {
        seq: [u8; N],
    }

    struct TextCycle<'a, const N: usize> {
        text: &'a Text<N>,
        index: usize,
    }

    impl<'a, const N: usize> Iterator for TextCycle<'a, N> {
        type Item = [u8; 8];

        fn next(&mut self) -> Option<Self::Item> {
            let res: [u8; 8] = self.text.seq[self.index..self.index + 8]
                .try_into()
                .unwrap();

            if self.index == 0 {
                self.index = N - 9;
            } else {
                self.index -= 1;
            }

            Some(res)
        }
    }

    impl<const N: usize> Text<N> {
        pub fn cycle(&self) -> TextCycle<N> {
            TextCycle {
                text: self,
                index: N - 15,
            }
        }
    }

    impl<const N: usize> From<[&[u8; 4]; N]> for Text<{ N * 4 + 8 }>
    // where N >= 2  // we always copy the last 2, but technically it should also be possible with 1
    {
        fn from(mut chars: [&[u8; 4]; N]) -> Self {
            let seq: [u8; N * 4 + 8] = unsafe {
                let mut res = core::mem::MaybeUninit::uninit();
                let res_ptr = res.as_mut_ptr() as *mut u8;

                let mut offset = 0;
                chars.reverse();
                for charr in chars {
                    core::ptr::copy_nonoverlapping(charr.as_ptr(), res_ptr.add(offset), 4);
                    offset += 4;
                }

                // copy last 2 again so it's easier to cycle
                core::ptr::copy_nonoverlapping(chars[0].as_ptr(), res_ptr.add(offset), 4);
                offset += 4;
                core::ptr::copy_nonoverlapping(chars[1].as_ptr(), res_ptr.add(offset), 4);
                // offset += 4;

                res.assume_init()
            };

            Text { seq }
        }
    }

    impl<const N: usize> From<&[u8; N]> for Text<{ N * 4 + 8 }> {
        fn from(chars: &[u8; N]) -> Self {
            use core::mem::MaybeUninit;

            let mut data: [MaybeUninit<&[u8; 4]>; N] =
                unsafe { MaybeUninit::uninit().assume_init() };

            let mut ix = 0;

            for chr in chars {
                let byt = match chr {
                    // TODO: macro expansion?
                    b'a' => &patterns::a,
                    b'b' => &patterns::b,
                    b'c' => &patterns::c,
                    b'd' => &patterns::d,
                    b'e' => &patterns::e,
                    b'f' => &patterns::f,
                    // b'g' => &patterns::g,
                    b'h' => &patterns::h,
                    b'i' => &patterns::i,
                    b'j' => &patterns::j,
                    // b'k' => &patterns::k,
                    b'l' => &patterns::l,
                    // b'm' => &patterns::m,
                    b'n' => &patterns::n,
                    b'o' => &patterns::o,
                    // b'p' => &patterns::p,
                    // b'q' => &patterns::q,
                    b'r' => &patterns::r,
                    // b's' => &patterns::s,
                    // b't' => &patterns::t,
                    b'u' => &patterns::u,
                    b'v' => &patterns::v,
                    b'w' => &patterns::w,
                    b'x' => &patterns::x,
                    // b'y' => &patterns::y,
                    // b'z' => &patterns::z,

                    // uppercase
                    // b'A' => &patterns::A,
                    // b'B' => &patterns::B,
                    b'C' => &patterns::C,
                    // b'D' => &patterns::D,
                    // b'E' => &patterns::E,
                    b'F' => &patterns::F,
                    // b'G' => &patterns::G,
                    // b'H' => &patterns::H,
                    // b'I' => &patterns::I,
                    // b'J' => &patterns::J,
                    // b'K' => &patterns::K,
                    // b'L' => &patterns::L,
                    // b'M' => &patterns::M,
                    // b'N' => &patterns::N,
                    // b'O' => &patterns::O,
                    // b'P' => &patterns::P,
                    // b'Q' => &patterns::Q,
                    // b'R' => &patterns::R,
                    b'S' => &patterns::S,
                    // b'T' => &patterns::T,
                    // b'U' => &patterns::U,
                    // b'V' => &patterns::V,
                    // b'W' => &patterns::W,
                    // b'X' => &patterns::X,
                    // b'Y' => &patterns::Y,
                    // b'Z' => &patterns::Z,

                    // specials
                    b'!' => &patterns::excl,
                    b' ' => &patterns::blank,
                    x => unimplemented!("no mapping for {}", *x as char),
                };

                data[ix].write(byt);
                ix += 1;
            }

            // SAFETY: fasten your seatbelts, here be dragons
            // https://stackoverflow.com/a/55313858
            unsafe {
                core::mem::transmute_copy::<_, MaybeUninit<[&[u8; 4]; N]>>(&data).assume_init()
            }
            .into()
        }
    }

    for _ in 0..=7 {
        matrix.write_raw(0, &patterns::Chess)?;
        block!(main_countdown.wait())?;
        block!(main_countdown.wait())?;
        matrix.write_raw(0, &patterns::InvertChess)?;
        block!(main_countdown.wait())?;
        block!(main_countdown.wait())?;
    }

    let text: Text<_> = b"Feroxide!".into();
    for bytes in text.cycle() {
        matrix.write_raw(0, &bytes)?;
        block!(main_countdown.wait())?;
    }

    panic!("should cycle endlessly");
}

#[entry]
fn main() -> ! {
    _main().unwrap();
    panic!("main should never end")
}
