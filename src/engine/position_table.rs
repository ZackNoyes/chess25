
use crate::{my_board::MyBoard, Score};

// 2^27 is the maximum we can get with Vec's allocation
// I've scaled it down a bit since hte allocation does take quite a while,
// especially with the debug build
const TABLE_SIZE: usize = 1 << 23;

#[derive(Clone, Copy)]
struct Parameters {
    pub depth: u8,
    pub dead_moves: u8,
}

#[derive(Clone, Copy)]
struct Evaluation {
    pub parameters: Parameters,
    pub score: Score,
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
#[derive(Clone, Copy, PartialEq, Eq)]
struct Position {
    zobrist_hash: u64,
    pieces: [Option<(Piece, Color)>; 64],
    castle_rights: [CastleRights; 2],
    side_to_move: Color,
}

pub struct PositionTable {
    table: Box<[Option<Evaluation>]>,
    // Debug info
    items: usize,
    insert_attempts: u64,
    insert_additions: u64,
    insert_ignores: u64,
    insert_overwrites: u64,
    get_attempts: u64,
    get_blanks: u64,
    get_hits: u64,
    get_incorrects: u64,
}

impl PositionTable {

    pub fn new() -> PositionTable {
        let table = vec![None; TABLE_SIZE].into_boxed_slice();
        web_sys::console::log_1(&format!(
            "Position table size: {} MB",
            table.len() * std::mem::size_of::<Option<Evaluation>>() / 1000000
        ).into());
        PositionTable {
            table: table,
            items: 0,
            insert_attempts: 0,
            insert_additions: 0,
            insert_ignores: 0,
            insert_overwrites: 0,
            get_attempts: 0,
            get_blanks: 0,
            get_hits: 0,
            get_incorrects: 0,
        }
    }

    /// Insert a board into the position table if we don't already have
    /// something better
    pub fn insert(&mut self, board: &MyBoard, depth: u8, score: Score) {
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
    pub fn insert_both_colors(&mut self, board: &MyBoard, depth: u8, score: Score) {
        let new_params = Parameters {
            depth,
            dead_moves: board.get_dead_moves(),
        };
        let mut position = Position::from_board(board);
        self.insert_position(position, new_params, score);
        position.switch_side_to_move();
        self.insert_position(position, new_params, score);
    }

    /// Insert a position and score into the table if the new parameters are
    /// `not_worse_than` the existing parameters.
    fn insert_position(&mut self, position: Position, params: Parameters, score: Score) {

        self.insert_attempts += 1;
        
        if match self.table[position.as_index()] { // TODO: Detect when you should ignore the insert
            None => {
                self.insert_additions += 1;
                self.items += 1;
                true
            },
            Some(_) => {
                self.insert_overwrites += 1; true
            },
        } {
            self.table[position.as_index()] = Some(Evaluation {
                parameters: params,
                score,
            });
        }
    }

    /// Get the score of a board if we have an existing evaluation of this
    /// board. Needs to be mutable to update the debug info
    pub fn get(&mut self, board: &MyBoard, depth: u8) -> Option<Score> {

        self.get_attempts += 1;

        let params = Parameters {
            depth,
            dead_moves: board.get_dead_moves(),
        };

        let pos = Position::from_board(&board);

        match self.table[pos.as_index()] {
            Some(evaluation) if evaluation.parameters.better_than(&params) => { // TODO: Incorrect retrievals
                self.get_hits += 1;
                Some(evaluation.score)
            },
            _ => {
                self.get_blanks += 1;
                None
            },
        }
    }

    pub fn info(&mut self) -> String {
        format!("Position table with {}/{} entries ({}% full):\n\
            \tTotal insert attempts: {}\n\
            \t\tAdditions: {} ({}%)\n\
            \t\tOverwrites: {} ({}%)\n\
            \t\tIgnores: {} ({}%)\n\
            \tTotal get attempts: {}\n\
            \t\tHits: {} ({}%)\n\
            \t\tBlanks: {} ({}%)\n\
            \t\tIncorrects: {} ({}%)\n",
            self.items,
            self.table.len(),
            (100 * self.items) / self.table.len(),
            self.insert_attempts,
            self.insert_additions,
            (100 * self.insert_additions).checked_div(self.insert_attempts).unwrap_or(0),
            self.insert_overwrites,
            (100 * self.insert_overwrites).checked_div(self.insert_attempts).unwrap_or(0),
            self.insert_ignores,
            (100 * self.insert_ignores).checked_div(self.insert_attempts).unwrap_or(0),
            self.get_attempts,
            self.get_hits,
            (100 * self.get_hits).checked_div(self.get_attempts).unwrap_or(0),
            self.get_blanks,
            (100 * self.get_blanks).checked_div(self.get_attempts).unwrap_or(0),
            self.get_incorrects,
            (100 * self.get_incorrects).checked_div(self.get_attempts).unwrap_or(0),
        )
    }

    pub fn reset_debug_info(&mut self) {
        self.insert_attempts = 0;
        self.insert_additions = 0;
        self.insert_ignores = 0;
        self.insert_overwrites = 0;
        self.get_attempts = 0;
        self.get_hits = 0;
        self.get_blanks = 0;
        self.get_incorrects = 0;
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
            zobrist_hash: board.get_zobrist_hash(),
        }
    }
    pub fn switch_side_to_move(&mut self) {
        self.zobrist_hash ^= crate::zobrist::Zobrist::color();
        self.side_to_move = !self.side_to_move;
    }
    pub fn as_index(&self) -> usize {
        self.zobrist_hash as usize % TABLE_SIZE
    }
}