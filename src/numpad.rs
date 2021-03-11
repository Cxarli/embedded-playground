use core::fmt::Debug;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use nb::block;
use stm32f1xx_hal::{pac, prelude::*, timer};

pub struct Buttons {
    _p: (),
}
pub type Button = u16;
#[allow(non_upper_case_globals)]
impl Buttons {
    pub const None: Button = 0;

    pub const One: Button = 1 << 0;
    pub const Two: Button = 1 << 1;
    pub const Three: Button = 1 << 2;
    pub const A: Button = 1 << 3;

    pub const Four: Button = 1 << 4;
    pub const Five: Button = 1 << 5;
    pub const Six: Button = 1 << 6;
    pub const B: Button = 1 << 7;

    pub const Seven: Button = 1 << 8;
    pub const Eight: Button = 1 << 9;
    pub const Nine: Button = 1 << 10;
    pub const C: Button = 1 << 11;

    pub const Asterisk: Button = 1 << 12;
    pub const Zero: Button = 1 << 13;
    pub const Octothorpe: Button = 1 << 14;
    pub const D: Button = 1 << 15;
}

pub struct Numpad<I: InputPin, O: OutputPin> {
    r0: Option<O>,
    r1: Option<O>,
    r2: Option<O>,
    r3: Option<O>,
    c0: Option<I>,
    c1: Option<I>,
    c2: Option<I>,
    c3: Option<I>,
}

impl<I: InputPin, O: OutputPin> Numpad<I, O>
where
    I::Error: Debug,
    O::Error: Debug,
{
    pub fn new(
        r0: Option<O>,
        r1: Option<O>,
        r2: Option<O>,
        r3: Option<O>,
        c0: Option<I>,
        c1: Option<I>,
        c2: Option<I>,
        c3: Option<I>,
    ) -> Self {
        let mut new = Self {
            r0,
            r1,
            r2,
            r3,
            c0,
            c1,
            c2,
            c3,
        };
        new.init();
        new
    }

    fn init(&mut self) {
        if let Some(ref mut r) = self.r0 {
            r.set_low().unwrap();
        }
        if let Some(ref mut r) = self.r1 {
            r.set_low().unwrap();
        }
        if let Some(ref mut r) = self.r2 {
            r.set_low().unwrap();
        }
        if let Some(ref mut r) = self.r3 {
            r.set_low().unwrap();
        }
    }

    fn scan_row(
        &mut self,
        row: u8,
        timer: &mut timer::CountDownTimer<pac::TIM1>,
        row_buttons: [Button; 4],
    ) -> Button {
        let mut buttons = Buttons::None;

        match row {
            0 => {
                if let Some(ref mut r) = self.r0 {
                    r.set_high().unwrap();
                } else {
                    return buttons;
                }
            }

            1 => {
                if let Some(ref mut r) = self.r1 {
                    r.set_high().unwrap();
                } else {
                    return buttons;
                }
            }

            2 => {
                if let Some(ref mut r) = self.r2 {
                    r.set_high().unwrap();
                } else {
                    return buttons;
                }
            }

            3 => {
                if let Some(ref mut r) = self.r3 {
                    r.set_high().unwrap();
                } else {
                    return buttons;
                }
            }

            _ => panic!(),
        };

        timer.start(500.ms());
        block!(timer.wait()).unwrap();

        if let Some(ref mut c) = self.c0 {
            if c.is_high().unwrap_or(false) {
                buttons |= row_buttons[0];
            }
        }
        if let Some(ref mut c) = self.c1 {
            if c.is_high().unwrap_or(false) {
                buttons |= row_buttons[1];
            }
        }
        if let Some(ref mut c) = self.c2 {
            if c.is_high().unwrap_or(false) {
                buttons |= row_buttons[2];
            }
        }
        if let Some(ref mut c) = self.c3 {
            if c.is_high().unwrap_or(false) {
                buttons |= row_buttons[3];
            }
        }

        match row {
            0 => {
                if let Some(ref mut r) = self.r0 {
                    r.set_low().unwrap();
                }
            }

            1 => {
                if let Some(ref mut r) = self.r1 {
                    r.set_low().unwrap();
                }
            }

            2 => {
                if let Some(ref mut r) = self.r2 {
                    r.set_low().unwrap();
                }
            }

            3 => {
                if let Some(ref mut r) = self.r3 {
                    r.set_low().unwrap();
                }
            }

            _ => panic!(),
        };

        buttons
    }

    pub fn read(&mut self, countdown: &mut timer::CountDownTimer<pac::TIM1>) -> Button {
        let mut buttons = Buttons::None;

        buttons |= self.scan_row(
            0,
            countdown,
            [Buttons::One, Buttons::Two, Buttons::Three, Buttons::A],
        );
        buttons |= self.scan_row(
            1,
            countdown,
            [Buttons::Four, Buttons::Five, Buttons::Six, Buttons::B],
        );
        buttons |= self.scan_row(
            2,
            countdown,
            [Buttons::Seven, Buttons::Eight, Buttons::Nine, Buttons::C],
        );
        buttons |= self.scan_row(
            3,
            countdown,
            [
                Buttons::Asterisk,
                Buttons::Zero,
                Buttons::Octothorpe,
                Buttons::D,
            ],
        );

        buttons
    }
}
