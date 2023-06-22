
use chess::{Piece, Color, CastleRights};

use crate::my_board::MyBoard;

/// A position is a representation of a game state. It contains the necessary
/// information to distinguish the state from other states, with the exception
/// of the player to move and the number of dead moves.
/// 
/// That is, it contains the positions of all pieces and the castling rights.
/// 
/// Note that en passant is not implemented, so it isn't included in the state
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    pieces: [Option<(Piece, Color)>; 64],
    castle_rights: [CastleRights; 2],
}

impl Position {
    pub fn from_my_board(board: &MyBoard) -> Position {
        todo!()
    }
}