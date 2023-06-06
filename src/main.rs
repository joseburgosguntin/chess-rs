#![feature(result_option_inspect)]

mod consts;
mod location;
mod piece;

use anyhow::bail;
use consts::{BYTES, LEN, SLOT_WIDTH};
use location::{Delta, Location, Number};
use piece::{Piece, PieceType, Slot, Team};
use std::io::{Stdin, Stdout, Write};

use thiserror::Error;

fn main() {
    // print!("\u{1b}[41m\u{1b}[37mhey");
    // print!("\u{1b}[41m\u{1b}[35mhey");
    // print!("\u{1b}[42m\u{1b}[37mhey");
    // print!("\u{1b}[42m\u{1b}[35mhey");
    // print!("\u{1b}[40m\u{1b}[37mhey");
    // print!("\u{1b}[40m\u{1b}[35mhey");
    // // print!("\u{1b}[43m\u{1b}[37mhey");
    // // print!("\u{1b}[43m\u{1b}[35mhey");
    // print!("\u{1b}[44m\u{1b}[37mhey");
    // print!("\u{1b}[44m\u{1b}[35mhey");
    // // print!("\u{1b}[45m\u{1b}[37mhey");
    // // print!("\u{1b}[45m\u{1b}[35mhey");
    // // print!("\u{1b}[46m\u{1b}[37mhey");
    // // print!("\u{1b}[46m\u{1b}[35mhey");
    // // print!("\u{1b}[47m\u{1b}[37mhey");
    // // print!("\u{1b}[47m\u{1b}[35mhey");
    // print!("\u{1b}[49m\u{1b}[37mhey");
    // print!("\u{1b}[49m\u{1b}[35mhey");
    // println!()
    // let mut chess = ChessBytes::new();
    // chess.play();
}

struct ChessBytes(*mut u8);

impl ChessBytes {
    fn new() -> Self {
        Self(unsafe { BYTES.as_mut_ptr() })
    }

    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.0, LEN) }
    }

    fn _as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }

    fn print(&self, stdout: &mut Stdout) {
        _ = stdout.write_all(self.as_bytes()).unwrap();
        _ = stdout.write_all(b"\n").unwrap();
    }

    fn get_bytes(&self, location: Location) -> [u8; 9] {
        unsafe { std::slice::from_raw_parts(self.0.offset(location.into()), 9) }
            .try_into()
            .unwrap()
    }

    /// NOT the default way of impl get, returns [`Slot`] instead of [`&Slot`](Slot)
    fn get(&self, location: Location) -> Slot {
        Slot::from(self.get_bytes(location))
    }

    fn play(&mut self) {
        use Team::*;
        let mut turn = White;
        let mut buf = String::with_capacity(10);
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        // todo!("move all the reused variables here");
        loop {
            _ = stdout.write_all("It's ".as_bytes()).unwrap();
            _ = match turn {
                Team::White => stdout.write_all("White's ".as_bytes()).unwrap(),
                Team::Black => stdout.write_all("Black's ".as_bytes()).unwrap(),
            };
            _ = stdout.write_all(" turn\n".as_bytes()).unwrap();
            _ = stdout
                .write_all("Chose the starting point\n".as_bytes())
                .unwrap();
            self.print(&mut stdout);
            loop {
                match self.try_move(&stdin, &mut stdout, &mut buf, turn) {
                    Ok(()) => break,
                    Err(e) => {
                        println!("{e}");
                    }
                }
            }
            self.print(&mut stdout);
            match turn {
                White => turn = Black,
                Black => turn = White,
            }
        }
    }

    fn try_move(
        &mut self,
        stdin: &Stdin,
        stdout: &mut Stdout,
        buf: &mut String,
        turn: Team,
    ) -> anyhow::Result<()> {
        buf.clear();
        let Ok(_) = stdin.read_line(buf) else {
            bail!("Failed to read line")
        };

        let [first, second, ..] = buf.as_bytes() else {
            bail!("Two characters need to specify starting point")
        };
        let location_1 = Location::try_from([*first, *second])?;

        let Slot::Piece(piece_1) = self.get(location_1) else {
            bail!("Empty tile")
        };
        if piece_1.team != turn {
            bail!("Not your piece")
        }

        self.highlight_board(location_1, piece_1, turn);

        stdout.write_all("Chose the ending point\n".as_bytes())?;
        buf.clear();
        let Ok(_) = stdin.read_line(buf) else {
            bail!("Failed to read line")
        };

        let [first, second, ..] = buf.as_bytes() else {
            bail!("Two characters need to specify destination point")
        };
        let location_2 = Location::try_from([*first, *second])?;

        match self.get_highlight(location_2, piece_1) {
            TryMove::Move => {}
            TryMove::Err(e) => return anyhow::Result::Err(e.into()),
        }

        // if let Slot::Piece(piece_2) = self.get(location_2) {
        //     // you can add the king rook exeption here before err
        //     if piece_2.team == turn {
        //         bail!("You can't move on a piece you own")
        //     }
        // }
        // if let Err(e) = self.try_delta(location_1, location_2, piece_1) {
        //     bail!("{e}")
        // }

        const NONE_STR: &str = "\u{1b}[49m\u{1b}[39m ​";

        // only 2 bytes need to actually be changed
        unsafe {
            std::ptr::copy_nonoverlapping(
                (self.0 as *const u8).offset(location_1.into()),
                self.0.offset(location_2.into()),
                SLOT_WIDTH,
            );
            std::ptr::copy_nonoverlapping(
                NONE_STR.as_ptr(),
                self.0.offset(location_1.into()),
                SLOT_WIDTH,
            );
        }

        return Ok(());
    }

    fn get_highlight(&self, location: Location, piece: Piece) -> TryMove {
        let base_offset: isize = location.into();
        TryMove::from((
            unsafe { std::ptr::read(self.0.offset(base_offset - 5 + 3)) },
            piece.team,
            piece.piece_type,
            location,
        ))
    }

    fn set_highlight(&mut self, location: Location, turn: Team) {
        let try_move = match self.get(location) {
            Slot::Piece(piece) => {
                if piece.team != turn {
                    TryMove::Move
                } else {
                    TryMove::Err(MoveErr::FriendlyFire(turn))
                }
            }
            Slot::Empty => TryMove::Move,
        };
        let base_offset: isize = location.into();
        unsafe { *self.0.offset(base_offset - 5 + 3) = try_move.into() };
    }

    fn set_highlight_with(&mut self, location: Location, try_move: TryMove) {
        let base_offset: isize = location.into();
        unsafe { *self.0.offset(base_offset - 5 + 3) = try_move.into() };
    }

    fn set_highlight_line(&mut self, location: Location, delta: Delta, turn: Team) {
        let mut maybe_location = Some(location);
        let mut obstructed = location;
        while Some(location) = maybe_location {
            if let Slot::Piece(_) = self.get(location) {
                break;
            }
            self.set_highlight(location, turn);
            obstructed = location;
            maybe_location = location + delta;
        }
        while Some(location) = maybe_location {
            self.set_highlight_with(location, TryMove::Err(MoveErr::PathObstructed(obstructed)));
            maybe_location = location + delta;
        }
    }

    fn highlight_board(&mut self, location: Location, piece: Piece, turn: Team) {
        use piece::PieceType::*;
        use MoveErr::*;
        use TryMove::*;
        match piece.piece_type {
            King => {
                [-1, 0, 1].into_iter().map(|x| {
                    [-1, 0, 1]
                        .into_iter()
                        .filter(|y| x == 0 && *y == 0)
                        .filter_map(|y| location + Delta(x, y))
                        .for_each(|l_2| self.set_highlight(l_2, turn))
                });
            }
            Queen => {
                [-1, 0, 1].into_iter().map(|x| {
                    [-1, 0, 1]
                        .into_iter()
                        .filter(|y| x == 0 && *y == 0)
                        .for_each(|y| self.set_highlight_line(location, Delta(x, y), turn))
                });
            }
            Bishop => {
                [-1, 1].into_iter().map(|x| {
                    [-1, 1]
                        .into_iter()
                        .for_each(|y| self.set_highlight_line(location, Delta(x, y), turn))
                });
            }
            Rook => {
                [-1, 1].into_iter().map(|s| {
                    [(0, 1), (1, 0)].into_iter().for_each(|(x, y)| {
                        self.set_highlight_line(location, Delta(x * s, y * s), turn)
                    })
                });
            }
            Knight => {
                [-1, 1].into_iter().map(|s_x| {
                    [-1, 1].into_iter().map(|s_y| {
                        [(1, 2), (2, 1)]
                            .into_iter()
                            .filter_map(|(x, y)| location + Delta(x * s_x, y * s_y))
                            .for_each(|l_2| {
                                self.set_highlight(l_2, turn);
                            })
                    })
                });
            }
            Pawn => {
                if !self.get(location).is_piece() {
                    if let Some(l_2) = location + Delta(0, 1 * turn.forward_direction()) {
                        self.set_highlight_with(l_2, TryMove::Move)
                    }
                    if let Some(l_2) = location + Delta(0, 2 * turn.forward_direction()) {
                        if (piece.team == Team::White && location.1 == Number::Two)
                            || (piece.team == Team::Black && location.1 == Number::Seven)
                        {
                            self.set_highlight_with(l_2, Move)
                        }
                    }
                } else {
                    [1, 2].into_iter().for_each(|s| {
                        if let Some(l_2) = location + Delta(0, s * turn.forward_direction()) {
                            self.set_highlight_with(l_2, Err(PathObstructed(l_2)))
                        }
                    });
                }

                [-1, 1].into_iter().for_each(|s| {
                    if let Some(l_2) = location + Delta(s, 1 * turn.forward_direction()) {
                        if let Slot::Piece(piece) = self.get(l_2) {
                            if piece.team != turn {
                                self.set_highlight_with(l_2, Move)
                            }
                        }
                    }
                });
            }
        };
    }

    // fn try_delta(&self, l_i: Location, l_f: Location, piece: Piece) -> anyhow::Result<()> {
    //     let delta = l_f - l_i;
    //     use PieceType::*;
    //     match piece.piece_type {
    //         King => match (delta.0.abs(), delta.1.abs()) {
    //             (0, 1) | (1, 0) | (1, 1) => Ok(()),
    //             _ => bail!("King can't move there"),
    //         },
    //         Queen => {
    //             if delta.0 == 0 || delta.1 == 0 || delta.0.abs() == delta.1.abs() {
    //                 let dir = delta.signum();
    //                 let mut loc = l_i + dir;
    //                 while loc != l_f {
    //                     if self.get(loc).is_piece() {
    //                         bail!("Queen's path is obstructed")
    //                     }
    //                     loc += dir;
    //                 }
    //                 Ok(())
    //             } else {
    //                 bail!("Queen can't move there")
    //             }
    //         }
    //         Bishop => {
    //             if delta.0.abs() == delta.1.abs() {
    //                 let dir = delta.signum();
    //                 let mut loc = l_i + dir;
    //                 while loc != l_f {
    //                     if self.get(loc).is_piece() {
    //                         bail!("Bishop's path is obstructed")
    //                     }
    //                     loc += dir;
    //                 }
    //                 Ok(())
    //             } else {
    //                 bail!("Bishop can't move there")
    //             }
    //         }
    //         Knight => match (delta.0.abs(), delta.1.abs()) {
    //             (1, 2) | (2, 1) => Ok(()),
    //             _ => bail!("Knight can't move there"),
    //         },
    //         Rook => {
    //             if delta.0 == 0 || delta.1 == 0 {
    //                 let dir = delta.signum();
    //                 let mut loc = l_i + dir;
    //                 while loc != l_f {
    //                     if self.get(loc).is_piece() {
    //                         bail!("Rook's path is obstructed")
    //                     }
    //                     loc += dir;
    //                 }
    //                 Ok(())
    //             } else {
    //                 bail!("Rook can't move there")
    //             }
    //         }
    //         Pawn => {
    //             // Check if the pawn is in its starting position
    //             if delta.0 == 0 && delta.1 == piece.team.forward_direction() {
    //                 if self.get(l_f).is_empty() {
    //                     Ok(())
    //                 } else {
    //                     bail!("Pawn can't move there")
    //                 }
    //             } else if delta.0 == 0
    //                 && delta.1 == 2 * piece.team.forward_direction()
    //                 && ((piece.team == Team::White && l_i.1 as i8 == 1)
    //                     || (piece.team == Team::Black && l_i.1 as i8 == 6))
    //             // Check if the pawn is in its starting position
    //             {
    //                 let intermediate_location = Location(
    //                     l_i.0,
    //                     Number::n(l_i.1 as i8 + piece.team.forward_direction()).unwrap(),
    //                 );
    //
    //                 if self.get(l_f).is_empty() && self.get(intermediate_location).is_empty() {
    //                     Ok(())
    //                 } else {
    //                     bail!("Pawn can't move there")
    //                 }
    //             } else if delta.0.abs() == 1 && delta.1 == piece.team.forward_direction() {
    //                 if let Slot::Piece(captured_piece) = self.get(l_f) {
    //                     if captured_piece.team != piece.team {
    //                         return Ok(());
    //                     } else {
    //                         bail!("Pawn can't capture its own piece")
    //                     }
    //                 } else {
    //                     bail!("Pawn can't move there")
    //                 }
    //             } else {
    //                 bail!("Invalid move for pawn")
    //             }
    //         }
    //     }
    // }
}

// ahhhhhh format anyhow macro's were also formated but whatever this is drier
// btw check for errors in the order I write em here
#[derive(Error, Debug)]
enum MoveErr {
    #[error("Moving there puts your king in check")]
    PutsYouInCheck, // red
    #[error("That's a {0} piece, it's on your team")]
    FriendlyFire(Team), // yellow
    #[error("Not a move in {0}'s moveset in chest")]
    NotInMoveSet(PieceType), // default
    #[error("Your path is being obstructed by a piece at {0}")]
    PathObstructed(Location), // cyan
}

#[derive(Error, Debug)]
enum SelectErr {
    #[error("That's a {0} piece, it's not on your")]
    NotYourPiece(Team),
    #[error("You're currently in check, move your king instead")]
    YoureInCheck,
}

enum TryMove {
    Move,
    Err(MoveErr),
}

impl From<(u8, Team, PieceType, Location)> for TryMove {
    fn from(value: (u8, Team, PieceType, Location)) -> Self {
        use MoveErr::*;
        use TryMove::*;
        match value.0 {
            0x31 => Err(PutsYouInCheck),
            0x32 => Err(FriendlyFire(value.1)),
            0x39 => Err(NotInMoveSet(value.2)),
            0x30 => Err(PathObstructed(value.3)),
            0x34 => Move,
            _ => unreachable!("Always valid background color"),
        }
    }
}

impl Into<u8> for TryMove {
    fn into(self) -> u8 {
        use MoveErr::*;
        use TryMove::*;
        match self {
            Err(PutsYouInCheck) => 0x31,
            Err(FriendlyFire(_)) => 0x32,
            // Err(NotInMoveSet(_)) => 0x39,
            Err(PathObstructed(_)) => 0x30,
            Move => 0x34,
            _ => unreachable!("Always valid background color"),
        }
    }
}

// impl Index<Location> for ChessBoard {
//     type Output = Option<Piece>;
//     fn index(&self, index: Location) -> &Self::Output {
//         &self.0[index.1 as usize][index.0 as usize]
//     }
// }
//
// impl IndexMut<Location> for ChessBoard {
//     fn index_mut(&mut self, index: Location) -> &mut Self::Output {
//         &mut self.0[index.1 as usize][index.0 as usize]
//     }
// }
//
// const HEIGHT: usize = Letter::COUNT;
//
// const WIDTH: usize = Number::COUNT;
//
// struct ChessBoard([[Option<Piece>; WIDTH]; HEIGHT], String);
//
// impl ChessBoard {
//     fn new() -> Self {
//         use PieceType::*;
//         use Team::*;
//         let b = [
//             [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook]
//                 .map(|piece_type| Some(Piece::new(White, piece_type))),
//             [Some(Piece::new(White, Pawn)); WIDTH],
//             [None; WIDTH],
//             [None; WIDTH],
//             [None; WIDTH],
//             [None; WIDTH],
//             [Some(Piece::new(Black, Pawn)); WIDTH],
//             [Rook, Knight, Bishop, Queen, King, Bishop, Knight, Rook]
//                 .map(|piece_type| Some(Piece::new(Black, piece_type))),
//         ];
//         Self(b, Self::new_string(&b))
//     }
//     fn new_string(board: &[[Option<Piece>; WIDTH]; HEIGHT]) -> String {
//         let mut string = String::with_capacity(((2 + (24 * WIDTH) + 1) * HEIGHT) + 17);
//         let mut char_buf = [0; 1];
//         for (i, number_line) in board.iter().enumerate() {
//             string.push_str(((49u8 + i as u8) as char).encode_utf8(&mut char_buf));
//             string.push_str(" ");
//             for slot in number_line {
//                 match slot {
//                     Some(piece) => string.push_str(piece.as_str()),
//                     None => string.push_str(NONE_STR),
//                 };
//             }
//             string.push_str("\u{1b}[39m\u{1b}[49m\n")
//         }
//         string.push_str("  a b c d e f g h");
//         string
//     }
//     fn play(&mut self) {
//         use Team::*;
//         let stdin = std::io::stdin();
//         let mut turn = White;
//         let mut buf = String::with_capacity(10); // hoping to avoid alloc
//         let mut char_buf = [0; 1];
//         loop {
//             loop {
//                 println!("{self}");
//                 println!("It's {turn:?}'s turn");
//                 let res = try_move(&stdin, &mut buf, &mut char_buf, self, turn);
//                 buf.clear();
//                 match res {
//                     Ok(_) => break,
//                     Err(_) => println!("Try again"),
//                 };
//             }
//             match turn {
//                 White => turn = Black,
//                 Black => turn = White,
//             }
//         }
//     }
// }
//
// fn try_move(
//     stdin: &Stdin,
//     buf: &mut String,
//     char_buf: &mut [u8],
//     board: &mut ChessBoard,
//     turn: Team,
// ) -> anyhow::Result<()> {
//     let (l_i, piece) = starting_point(stdin, buf, char_buf, board, turn)?;
//     buf.clear();
//     let l_f = destination(stdin, buf, char_buf, board, turn)?;
//     buf.clear();
//     // There should be a select lighlight before the mutation
//     logic(piece, board, l_i, l_f)?;
//     // using format is expensive re-render
//     Ok(())
// }
//
// fn starting_point(
//     stdin: &Stdin,
//     buf: &mut String,
//     char_buf: &mut [u8],
//     board: &ChessBoard,
//     team: Team,
// ) -> anyhow::Result<(Location, Piece)> {
//     stdin.read_line(buf)?;
//     let bytes = buf.as_bytes();
//     let [first, second, ..] = bytes else {
//         let e = anyhow!("Failed to get first two elements");
//         println!("{e}");
//         return Err(e);
//     };
//     let (letter, number) = (
//         Letter::from_str((*first as char).encode_utf8(char_buf)).map_err(|_| {
//             let e = anyhow!("First character isn't a letter in chess");
//             println!("{e}");
//             e
//         })?,
//         Number::from_str((*second as char).encode_utf8(char_buf)).map_err(|_| {
//             let e = anyhow!("Second character isn't a number in chess");
//             println!("{e}");
//             e
//         })?,
//     );
//     let location = Location(letter, number);
//     let Some(piece) = board[location] else {
//         let e = anyhow!("There is no piece there");
//         println!("{e}");
//         return Err(e);
//     };
//     if piece.team != team {
//         let e = anyhow!("That piece is not of your team");
//         println!("{e}");
//         return Err(e);
//     }
//     Ok((location, piece))
// }
//
// fn destination(
//     stdin: &Stdin,
//     buf: &mut String,
//     char_buf: &mut [u8],
//     board: &ChessBoard,
//     team: Team,
// ) -> anyhow::Result<Location> {
//     stdin.read_line(buf)?;
//     let bytes = buf.as_bytes();
//     let [first, second, ..] = bytes else {
//         let e = anyhow!("Failed to get first two elements");
//         println!("{e}");
//         return Err(e);
//     };
//     let (letter, number) = (
//         Letter::from_str((*first as char).encode_utf8(char_buf)).map_err(|_| {
//             let e = anyhow!("First character isn't a letter in chess");
//             println!("{e}");
//             e
//         })?,
//         Number::from_str((*second as char).encode_utf8(char_buf)).map_err(|_| {
//             let e = anyhow!("Second character isn't a number in chess");
//             println!("{e}");
//             e
//         })?,
//     );
//     let location = Location(letter, number);
//     if let Some(piece) = board[location] {
//         if piece.team == team {
//             let e = anyhow!("That piece is on your team");
//             println!("{e}");
//             return Err(e);
//         }
//     }
//     Ok(location)
// }
// impl Add<Delta> for Location {
//     type Output = Location;
//     fn add(self, rhs: Delta) -> Self::Output {
//         Location(
//             Letter::n(self.0 as i8 + rhs.0).unwrap(),
//             Number::n(self.1 as i8 + rhs.1).unwrap(),
//         )
//     }
// }

// fn logic(piece: Piece, board: &mut ChessBoard, l_i: Location, l_f: Location) -> anyhow::Result<()> {
//     let delta = l_f - l_i;
//     use PieceType::*;
//     match piece.piece_type {
//         King => match (delta.0.abs(), delta.1.abs()) {
//             (0, 1) | (1, 0) | (1, 1) => {
//                 board[l_f] = Some(piece);
//                 board[l_i] = None;
//                 Ok(())
//             }
//             _ => {
//                 let e = anyhow!("King can't move there");
//                 println!("{:?}", e);
//                 Err(e)
//             }
//         },
//         Queen => {
//             if delta.0 == 0 || delta.1 == 0 || delta.0.abs() == delta.1.abs() {
//                 let dir = delta.signum();
//                 let mut loc = l_i + dir;
//                 while loc != l_f {
//                     if board[loc].is_some() {
//                         let e = anyhow!("Queen's path is obstructed");
//                         println!("{:?}", e);
//                         return Err(e);
//                     }
//                     loc += dir;
//                 }
//                 board[l_f] = Some(piece);
//                 board[l_i] = None;
//                 Ok(())
//             } else {
//                 let e = anyhow!("Queen can't move there");
//                 println!("{:?}", e);
//                 Err(e)
//             }
//         }
//         Bishop => {
//             if delta.0.abs() == delta.1.abs() {
//                 let dir = delta.signum();
//                 let mut loc = l_i + dir;
//                 while loc != l_f {
//                     if board[loc].is_some() {
//                         let e = anyhow!("Bishop's path is obstructed");
//                         println!("{:?}", e);
//                         return Err(e);
//                     }
//                     loc += dir;
//                 }
//                 board[l_f] = Some(piece);
//                 board[l_i] = None;
//                 Ok(())
//             } else {
//                 let e = anyhow!("Bishop can't move there");
//                 println!("{:?}", e);
//                 Err(e)
//             }
//         }
//         Knight => match (delta.0.abs(), delta.1.abs()) {
//             (1, 2) | (2, 1) => {
//                 board[l_f] = Some(piece);
//                 board[l_i] = None;
//                 Ok(())
//             }
//             _ => {
//                 let e = anyhow!("Knight can't move there");
//                 println!("{:?}", e);
//                 Err(e)
//             }
//         },
//         Rook => {
//             if delta.0 == 0 || delta.1 == 0 {
//                 let dir = delta.signum();
//                 let mut loc = l_i + dir;
//                 while loc != l_f {
//                     if board[loc].is_some() {
//                         let e = anyhow!("Rook's path is obstructed");
//                         println!("{:?}", e);
//                         return Err(e);
//                     }
//                     loc += dir;
//                 }
//                 board[l_f] = Some(piece);
//                 board[l_i] = None;
//                 Ok(())
//             } else {
//                 let e = anyhow!("Rook can't move there");
//                 println!("{:?}", e);
//                 Err(e)
//             }
//         }
//         Pawn => {
//             // Check if the pawn is in its starting position
//             if delta.0 == 0 && delta.1 == piece.team.forward_direction() {
//                 if board[l_f].is_none() {
//                     board[l_f] = Some(piece);
//                     board[l_i] = None; // Set the original location to None
//                     Ok(())
//                 } else {
//                     let e = anyhow!("Pawn can't move there");
//                     println!("{:?}", e);
//                     Err(e)
//                 }
//             } else if delta.0 == 0
//                 && delta.1 == 2 * piece.team.forward_direction()
//                 && ((piece.team == Team::White && l_i.1 as i8 == 1)
//                     || (piece.team == Team::Black && l_i.1 as i8 == 6))
//             // Check if the pawn is in its starting position
//             {
//                 let intermediate_location = Location(
//                     l_i.0,
//                     Number::n(l_i.1 as i8 + piece.team.forward_direction()).unwrap(),
//                 );
//
//                 if board[l_f].is_none() && board[intermediate_location].is_none() {
//                     board[l_f] = Some(piece);
//                     board[l_i] = None; // Set the original location to None
//                     Ok(())
//                 } else {
//                     let e = anyhow!("Pawn can't move there");
//                     println!("{:?}", e);
//                     Err(e)
//                 }
//             } else if delta.0.abs() == 1 && delta.1 == piece.team.forward_direction() {
//                 if let Some(captured_piece) = board[l_f] {
//                     if captured_piece.team != piece.team {
//                         board[l_f] = Some(piece);
//                         board[l_i] = None; // Set the original location to None
//                         return Ok(());
//                     } else {
//                         let e = anyhow!("Pawn can't capture its own piece");
//                         println!("{:?}", e);
//                         return Err(e);
//                     }
//                 } else {
//                     let e = anyhow!("Pawn can't move there");
//                     println!("{:?}", e);
//                     return Err(e);
//                 }
//             } else {
//                 let e = anyhow!("Invalid move for pawn");
//                 println!("{:?}", e);
//                 return Err(e);
//             }
//         }
//     }
// }
//
// impl Display for ChessBoard {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let mut string = String::with_capacity(((2 + (24 * WIDTH) + 1) * HEIGHT) + 17);
//         let mut char_buf = [0; 1];
//         for (i, number_line) in self.0.iter().enumerate() {
//             string.push_str(((49u8 + i as u8) as char).encode_utf8(&mut char_buf));
//             string.push_str(" ");
//             for slot in number_line {
//                 match slot {
//                     Some(piece) => string.push_str(piece.as_str()),
//                     None => string.push_str(w!(" ​")),
//                 };
//             }
//             string.push_str("\n")
//         }
//         string.push_str("  a b c d e f g h");
//         write!(f, "{}", string)
//     }
// }
