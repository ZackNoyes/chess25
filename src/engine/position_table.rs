use std::collections::HashMap;

use crate::my_board::MyBoard;

#[derive(Clone, Copy)]
struct Parameters {
    pub depth: u8,
    pub dead_moves: u8,
}

struct Evaluation {
    pub parameters: Parameters,
    pub score: f64,
}

use chess::{Piece, Color, CastleRights};

/// A position is a representation of a game state. It contains the necessary
/// information to distinguish the state from other states, with the exception
/// of the number of dead moves.
/// 
/// That is, it contains the positions of all pieces, the castling rights, and
/// the side to move.
/// 
/// Note that en passant is not implemented, so it isn't included in the state
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct Position {
    pieces: [Option<(Piece, Color)>; 64],
    castle_rights: [CastleRights; 2],
    side_to_move: Color,
}

pub struct PositionTable {
    table: HashMap<Position, Evaluation>,
    // Debug info
    insert_attempts: u64,
    insert_additions: u64,
    insert_ignores: u64,
    insert_overwrites: u64,
    get_attempts: u64,
    get_misses: u64,
    get_hits: u64,
}

impl PositionTable {
    pub fn new() -> PositionTable {
        PositionTable {
            table: HashMap::new(),
            insert_attempts: 0,
            insert_additions: 0,
            insert_ignores: 0,
            insert_overwrites: 0,
            get_attempts: 0,
            get_misses: 0,
            get_hits: 0,
        }
    }

    /// Insert a board into the position table if we don't already have
    /// something better
    pub fn insert(&mut self, board: &MyBoard, depth: u8, score: f64) {
        let new_params = Parameters {
            depth,
            dead_moves: board.get_dead_moves(),
        };
        let position = Position::from_board(board);
        self.insert_position(position, new_params, score);
    }

    /// Insert a board into the position table for both colors if we don't
    /// already have something better. This might be useful when the depth
    /// is 0 and so the evaluation is known to be the same for both colors.
    pub fn insert_both_colors(&mut self, board: &MyBoard, depth: u8, score: f64) {
        let new_params = Parameters {
            depth,
            dead_moves: board.get_dead_moves(),
        };
        let mut position = Position::from_board(board);
        self.insert_position(position, new_params, score);
        position.side_to_move = !position.side_to_move;
        self.insert_position(position, new_params, score);
    }

    /// Insert a position and score into the table if the new parameters are
    /// `not_worse_than` the existing parameters.
    fn insert_position(&mut self, position: Position, params: Parameters, score: f64) {

        self.insert_attempts += 1;
        
        if match self.table.get(&position) {
            None => {
                self.insert_additions += 1; true
            },
            Some(evaluation) if evaluation.parameters.not_worse_than(&params) => {
                self.insert_overwrites += 1; true
            },
            _ => {
                self.insert_ignores += 1; false
            }
        } {
            self.table.insert(position, Evaluation {
                parameters: params,
                score,
            });
        }
    }

    /// Get the score of a board if we have an existing evaluation of this
    /// board. Needs to be mutable to update the debug info
    pub fn get(&mut self, board: &MyBoard, depth: u8) -> Option<f64> {

        self.get_attempts += 1;

        let params = Parameters {
            depth,
            dead_moves: board.get_dead_moves(),
        };

        match self.table.get(&Position::from_board(&board)) {
            Some(evaluation) if evaluation.parameters.better_than(&params) => {
                self.get_hits += 1;
                Some(evaluation.score)
            },
            _ => {
                self.get_misses += 1;
                None
            },
        }
    }

    pub fn info(&mut self) -> String {
        format!("Position table of capacity {}:\n\
            \tTotal insert attempts: {}\n\
            \t\tAdditions: {} ({:.1}%)\n\
            \t\tOverwrites: {} ({:.1}%)\n\
            \t\tIgnores: {} ({:.1}%)\n\
            \tTotal get attempts: {}\n\
            \t\tHits: {} ({:.1}%)\n\
            \t\tMisses: {} ({:.1}%)\n",
            self.table.capacity(),
            self.insert_attempts,
            self.insert_additions,
            100 * self.insert_additions.checked_div(self.insert_attempts).unwrap_or(0),
            self.insert_overwrites,
            100 * self.insert_overwrites.checked_div(self.insert_attempts).unwrap_or(0),
            self.insert_ignores,
            100 * self.insert_ignores.checked_div(self.insert_attempts).unwrap_or(0),
            self.get_attempts,
            self.get_hits,
            100 * self.get_hits.checked_div(self.get_attempts).unwrap_or(0),
            self.get_misses,
            100 * self.get_misses.checked_div(self.get_attempts).unwrap_or(0),
        )
    }

    pub fn reset_debug_info(&mut self) {
        self.insert_attempts = 0;
        self.insert_additions = 0;
        self.insert_ignores = 0;
        self.insert_overwrites = 0;
        self.get_attempts = 0;
        self.get_hits = 0;
        self.get_misses = 0;
    }

}

impl Parameters {
    fn saw_50_move_rule(&self) -> bool {
        50 - self.dead_moves <= self.depth
    }
    pub fn not_worse_than(&self, other: &Self) -> bool {
        self.saw_50_move_rule()
        || other.saw_50_move_rule()
        || self.depth >= other.depth
    }
    pub fn better_than(&self, other: &Self) -> bool {
        self.depth >= other.depth && (
            self.dead_moves == other.dead_moves
            || (!self.saw_50_move_rule() && !other.saw_50_move_rule())
        )
    }
}

impl Position {
    pub fn from_board(board: &MyBoard) -> Position {
        Position {
            pieces: board.get_pieces(),
            castle_rights: board.get_castle_rights_arr(),
            side_to_move: board.get_side_to_move(),
        }
    }
}