use chess::{Color::*, Piece::*};
use serde::{Deserialize, Serialize};

use crate::{MyBoard, Score, StaticEvaluator};

/// Weights that are designed to be multiplied by corresponding features
/// using a dot product
#[derive(Clone, Copy)]
pub struct Weights {
    pub pieces: [[f32; 6]; 2],
    pub king_danger: [f32; 2],
    pub pawn_advancement: [f32; 2],
    pub side_to_move: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Features {
    /// The number of pieces of each type for each player
    pub pieces: [[f32; 6]; 2],
    /// The number of squares that each players' king could be attacked from
    pub king_danger: [f32; 2],
    /// The average rank of each players' pawns
    pub pawn_advancement: [f32; 2],
    /// Whose turn it is to move (1 for white, -1 for black)
    pub side_to_move: f32,
}

impl Features {
    pub fn from_board(board: &MyBoard) -> Features {
        assert!(board.get_status().is_in_progress());

        let mut pieces = [[0.0; 6]; 2];
        let mut king_danger = [0.0; 2];
        let mut pawn_advancement = [0.0; 2];

        for col in [White, Black] {
            let my_pieces = if col == White {
                board.get_white_pieces()
            } else {
                board.get_black_pieces()
            };
            let not_my_pieces = !my_pieces;

            for sq in if col == White {
                board.get_white_pieces()
            } else {
                board.get_black_pieces()
            } {
                let Some((piece, _)) = board[sq]
                    else { panic!("piece not found on square {:?}", sq); };
                pieces[col.to_index()][piece.to_index()] += 1.0;

                if piece == King {
                    king_danger[col.to_index()] += ((chess::get_knight_moves(sq)
                        | chess::get_bishop_moves(sq, my_pieces)
                        | chess::get_rook_moves(sq, my_pieces))
                        & not_my_pieces)
                        .popcnt() as f32;
                }

                if piece == Pawn {
                    pawn_advancement[col.to_index()] += if col == White {
                        sq.get_rank().to_index() as f32 - 1.0
                    } else {
                        6.0 - sq.get_rank().to_index() as f32
                    }
                }
            }
            let pawns = pieces[col.to_index()][Pawn.to_index()];
            pawn_advancement[col.to_index()] = if pawns != 0.0 {
                pawn_advancement[col.to_index()] / pawns
            } else {
                0.0
            };
        }

        let side_to_move = if board.get_side_to_move() == White {
            1.0
        } else {
            -1.0
        };

        Features {
            pieces,
            king_danger,
            pawn_advancement,
            side_to_move,
        }
    }
}

pub struct FeatureEval {
    weights: Weights,
    scale_down: f32,
}

impl StaticEvaluator for FeatureEval {
    fn evaluate(&self, board: &MyBoard) -> Score {
        if !board.get_status().is_in_progress() {
            return self.evaluate_terminal(board).unwrap();
        }

        let features = Features::from_board(board);

        let mut score: f32 = 0.0;

        for col in [White, Black] {
            for piece in [Pawn, Knight, Bishop, Rook, Queen, King] {
                score += self.weights.pieces[col.to_index()][piece.to_index()]
                    * features.pieces[col.to_index()][piece.to_index()];
            }
            score +=
                self.weights.king_danger[col.to_index()] * features.king_danger[col.to_index()];
            score += self.weights.pawn_advancement[col.to_index()]
                * features.pawn_advancement[col.to_index()];
        }

        let adjusted = Self::sigmoid(score / self.scale_down);

        Score::from_num(adjusted)
    }
}

impl FeatureEval {
    pub fn new(weights: Weights, scale_down: f32) -> FeatureEval {
        FeatureEval {
            weights,
            scale_down,
        }
    }

    fn sigmoid(x: f32) -> f32 { 1.0 / (1.0 + (-x).exp()) }
}
