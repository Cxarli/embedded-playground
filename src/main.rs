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
use stm32f1xx_hal::{pac, prelude::*, pwm, timer};

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
    (Void, void::Void),
    (Fmt, core::fmt::Error),
    (Infallible, Infallible),
    (I2c, stm32f1xx_hal::i2c::Error),
    (Max7219, max7219::DataError),
    (Owb, one_wire_bus::OneWireError<Infallible>),
);

/// Wrapper around main which supports returning errors
fn _main() -> core::result::Result<(), Error> {
    #[cfg(feature = "semi")]
    // connect stdout
    let mut stdout = hio::hstdout()?;

    // start timer
    let dev_peripherals = pac::Peripherals::take().ok_or(())?;
    let mut flash = dev_peripherals.FLASH.constrain();
    let mut radio_clock = dev_peripherals.RCC.constrain();
    let clocks = radio_clock.cfgr.freeze(&mut flash.acr);
    let tim2 = timer::Timer::tim2(dev_peripherals.TIM2, &clocks, &mut radio_clock.apb1);
    let tim3 = timer::Timer::tim3(dev_peripherals.TIM3, &clocks, &mut radio_clock.apb1);
    let mut main_countdown = tim2.start_count_down(150.ms());

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
        /*displays*/ 1, /* data = sda/sdi */ pb9, /* cs = chip select */ pb4,
        /* sck = clock */ pb8,
    )?;
    matrix.power_on()?;

    /*
    // speaker
    let mut pa6 = gpioa.pa6.into_alternate_push_pull(&mut gpioa.crl);
    let mut speaker = tim3.pwm(pa6, &mut afio.mapr, 300.ms());
    speaker.set_duty(pwm::Channel::C1, speaker.get_max_duty());
    speaker.enable(pwm::Channel::C1);
    */

    #[cfg(feature = "semi")]
    write!(stdout, "Hello, world!\n")?;

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
    panic!()
}
