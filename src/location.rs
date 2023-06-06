use anyhow::bail;
use enumn::N;
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Sub},
};
use strum_macros::{EnumCount as EnumCountMacro, EnumString};

const HORIZONTAL_COUNT: usize = 8;
const PIECE_WIDTH: usize = 4;
const ANSI_WIDTH: usize = 10;
const SLOT_WIDTH: usize = ANSI_WIDTH + PIECE_WIDTH;

#[derive(N, EnumCountMacro, EnumString, Debug, PartialEq, Clone, Copy)]
enum Letter {
    #[strum(ascii_case_insensitive)]
    A,
    #[strum(ascii_case_insensitive)]
    B,
    #[strum(ascii_case_insensitive)]
    C,
    #[strum(ascii_case_insensitive)]
    D,
    #[strum(ascii_case_insensitive)]
    E,
    #[strum(ascii_case_insensitive)]
    F,
    #[strum(ascii_case_insensitive)]
    G,
    #[strum(ascii_case_insensitive)]
    H,
}

#[derive(N, EnumCountMacro, EnumString, Debug, PartialEq, Clone, Copy)]
pub enum Number {
    #[strum(serialize = "1")]
    One,
    #[strum(serialize = "2")]
    Two,
    #[strum(serialize = "3")]
    Three,
    #[strum(serialize = "4")]
    Four,
    #[strum(serialize = "5")]
    Five,
    #[strum(serialize = "6")]
    Six,
    #[strum(serialize = "7")]
    Seven,
    #[strum(serialize = "8")]
    Eight,
}

#[derive(Clone, Copy)]
pub struct Delta(pub i8, pub i8);

impl Delta {
    fn signum(&self) -> Self {
        Delta(self.0.signum(), self.1.signum())
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Location(Letter, pub Number);

impl Sub<Location> for Location {
    type Output = Delta;
    fn sub(self, rhs: Location) -> Self::Output {
        Delta(self.0 as i8 - rhs.0 as i8, self.1 as i8 - rhs.1 as i8)
    }
}

impl Add<Delta> for Location {
    type Output = Option<Location>;
    fn add(self, rhs: Delta) -> Self::Output {
        Some(Location(
            Letter::n(self.0 as i8 + rhs.0)?,
            Number::n(self.1 as i8 + rhs.1)?,
        ))
    }
}

// sussy
impl AddAssign<Delta> for Location {
    fn add_assign(&mut self, rhs: Delta) {
        self.0 = Letter::n(self.0 as i8 + rhs.0).unwrap();
        self.1 = Number::n(self.1 as i8 + rhs.1).unwrap();
    }
}

impl Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}{:?}", self.0, self.1)
    }
}

impl Into<isize> for Location {
    fn into(self) -> isize {
        let Location(l, n) = self;
        let x_offset = (SLOT_WIDTH * (l as usize)) + "1 ".len() + 5;
        let y_offset =
            (n as usize) * ("1 ".len() + (SLOT_WIDTH * HORIZONTAL_COUNT) + ANSI_WIDTH + "\n".len());
        (x_offset + y_offset) as isize
    }
}

impl TryFrom<[u8; 2]> for Location {
    type Error = anyhow::Error;
    fn try_from(value: [u8; 2]) -> Result<Self, Self::Error> {
        let Some(letter) = (match value[0] {
            first @ b'A'..=b'H' => Letter::n(first - b'A'),
            first @ b'a'..=b'h' => Letter::n(first - b'a'),
            _ => bail!("Not a chess letter")
        }) else {
            unreachable!("Always vaild letter")
        };
        let Some(number) = (match value[1] {
            second @ b'1'..=b'8' => Number::n(second - b'1'),
            _ => bail!("Not a chess number")
        }) else {
            unreachable!("Always vaild number")
        };
        Ok(Location(letter, number))
    }
}
