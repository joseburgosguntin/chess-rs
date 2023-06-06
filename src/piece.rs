use std::fmt::Display;

use enumn::N;

//
// macro_rules! k {
//     ($s:literal) => {
//         concat!("\u{1b}[49m\u{1b}[35m", $s)
//     };
// }
//
// macro_rules! w {
//     ($s:literal) => {
//         concat!("\u{1b}[49m\u{1b}[37m", $s)
//     };
// }

#[derive(N, Clone, Copy, Debug)]
pub enum PieceType {
    King = 0x97,
    Knight,
    Pawn,
    Queen,
    Rook,
    Bishop,
}

impl Display for PieceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(N, Clone, Copy, Debug, PartialEq)]
pub enum Team {
    White = 0x37,
    Black = 0x35,
}

impl Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Team {
    pub fn forward_direction(&self) -> i8 {
        use Team::*;
        match self {
            White => 1,
            Black => -1,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub piece_type: PieceType,
    pub team: Team,
}

impl Piece {
    // fn new(team: Team, piece_type: PieceType) -> Self {
    //     Self { team, piece_type }
    // }
    // fn as_str(self) -> &'static str {
    //     use PieceType::*;
    //     use Team::*;
    //     match self.team {
    //         White => match self.piece_type {
    //             King => w!("󰡗"),
    //             Queen => w!("󰡚"),
    //             Rook => w!("󰡛"),
    //             Bishop => w!("󰡜"),
    //             Knight => w!("󰡘"),
    //             Pawn => w!("󰡙"),
    //         },
    //         Black => match self.piece_type {
    //             King => k!("󰡗"),
    //             Queen => k!("󰡚"),
    //             Rook => k!("󰡛"),
    //             Bishop => k!("󰡜"),
    //             Knight => k!("󰡘"),
    //             Pawn => k!("󰡙"),
    //         },
    //     }
    // }
}

// impl Display for Piece {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.as_str())
//     }
// }

pub enum Slot {
    Piece(Piece),
    Empty,
}

impl Slot {
    pub fn is_piece(&self) -> bool {
        match self {
            Slot::Piece(_) => true,
            Slot::Empty => false,
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Slot::Piece(_) => false,
            Slot::Empty => true,
        }
    }
}

impl From<[u8; 9]> for Slot {
    fn from(value: [u8; 9]) -> Self {
        match value {
            // begining background bytes 0x1b, 0x5b, 0x34, 0x39, 0x6d
            // foreground 0x1b, 0x5b, 0x33, _ 0x6d
            [_, _, _, t @ (0x37 | 0x35), _, _, _, _, p @ 0x97..=0x9c] => Slot::Piece(Piece {
                team: Team::n(t).expect("Always valid team penultimate byte"),
                piece_type: PieceType::n(p).expect("Always valid piece type last byte"),
            }),
            _ => Slot::Empty,
        }
    }
}
