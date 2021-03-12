use core::convert::Infallible;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use stm32f1xx_hal::gpio::{Input, Output, PullDown, PushPull, Pxx};

pub struct Buttons {
    /// make sure it can't be constructed
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

    pub const Star: Button = 1 << 12;
    pub const Zero: Button = 1 << 13;
    pub const Hash: Button = 1 << 14;
    pub const D: Button = 1 << 15;
}

/// The layout of the numpad
const LAYOUT: [[Button; 4]; 4] = [
    [Buttons::One, Buttons::Two, Buttons::Three, Buttons::A],
    [Buttons::Four, Buttons::Five, Buttons::Six, Buttons::B],
    [Buttons::Seven, Buttons::Eight, Buttons::Nine, Buttons::C],
    [Buttons::Star, Buttons::Zero, Buttons::Hash, Buttons::D],
];

type Out = Option<Pxx<Output<PushPull>>>;
type In = Option<Pxx<Input<PullDown>>>;

pub struct Numpad {
    rows: [Out; 4],
    cols: [In; 4],
}

impl Numpad {
    /// Create a new Numpad
    pub fn new<E: From<Infallible>>(mut rows: [Out; 4], cols: [In; 4]) -> Result<Self, E> {
        // Set all outputs low
        #[allow(clippy::manual_flatten)]
        for pin in rows.iter_mut() {
            if let Some(r) = pin {
                r.set_low()?;
            }
        }

        Ok(Self { rows, cols })
    }

    /// Get all active buttons on the given row index
    fn scan_row<E: From<Infallible>>(
        &mut self,
        row: usize,
        row_buttons: [Button; 4],
    ) -> Result<Button, E> {
        // Get the current row
        let row = &mut self.rows[row];
        if row.is_none() {
            return Ok(Buttons::None);
        }
        let row = row.as_mut().unwrap();

        // Enable the row
        row.set_high()?;

        // Check all columns
        let mut buttons = Buttons::None;
        for (i, col) in self.cols.iter_mut().enumerate() {
            if let Some(ref mut c) = col {
                if c.is_high().unwrap_or(false) {
                    buttons |= row_buttons[i];
                }
            }
        }

        // Reset row
        row.set_low()?;

        Ok(buttons)
    }

    /// Read the entire numpad
    pub fn read<E: From<Infallible>>(&mut self) -> Result<Button, E> {
        let mut buttons = Buttons::None;
        for (i, &layout) in LAYOUT.iter().enumerate() {
            buttons |= self.scan_row(i, layout)?;
        }

        Ok(buttons)
    }
}
