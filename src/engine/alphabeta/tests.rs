use chess::Color;

use super::*;
use crate::engine::{
    feature_eval::{FeatureEval, Weights},
    proportion_count::ProportionCount,
};

#[test]
#[ignore]
fn test_self_game() {
    let mut white = AlphaBeta::new(ProportionCount::default(), 2, false, true, 10, 1000);
    let mut black = AlphaBeta::new(ProportionCount::default(), 2, true, false, 10, 1000);

    let mut board = MyBoard::initial_board(Color::White);

    let mut moves = 0;

    loop {
        if !board.get_status().is_in_progress() {
            break;
        }

        let mv = if board.get_side_to_move() == Color::White {
            white.get_move(&board)
        } else {
            black.get_move(&board)
        };

        board.apply_move(mv);
        board.apply_bonus(moves % 5 == 0);
        moves += 1;
        println!("--------------------");
        println!("{}", board);

        let ws = Weights {
            pieces: [[1.0, 3.0, 3.0, 5.0, 9.0, 0.0], [
                -1.0, -3.0, -3.0, -5.0, -9.0, 0.0,
            ]],
            king_danger: [-0.5, 0.5],
            pawn_advancement: [1.0, -1.0],
            side_to_move: 3.0,
        };
        check_inversions(&board, || {
            AlphaBeta::new(ProportionCount::default(), 3, false, false, 0, 100000)
        });
        check_inversions(&board, || {
            AlphaBeta::new(ProportionCount::default(), 4, false, true, 0, 100000)
        });
        check_inversions(&board, || {
            AlphaBeta::new(FeatureEval::new(ws, 20.0), 3, false, false, 0, 100000)
        });
        check_inversions(&board, || {
            AlphaBeta::new(FeatureEval::new(ws, 20.0), 4, false, true, 0, 100000)
        });
    }
}

fn check_inversions(board: &MyBoard, engine_creator: impl Fn() -> AlphaBeta) {
    // Board 0 is the normal board.
    // Board 1 should match 0.
    // Board 2 is the normal board with castling rights stripped.
    // Board 3 should match 2.
    // Board 4 should match 2.

    // TODO: Fix the fact that the test fails by a bit

    let mut boards = [*board; 5];
    boards[1].invert_ranks_and_colors();
    boards[2].strip_castle_rights();
    boards[3].strip_castle_rights();
    boards[3].invert_files();
    boards[4].strip_castle_rights();
    boards[4].invert_files();
    boards[4].invert_ranks_and_colors();

    let results = boards
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let mut engine = engine_creator();
            let Result(sc1, _) = engine.get_scored_best_move(
                b, Bounds::widest(), engine.max_lookahead, false, Deadline::from_now(100000)
            )
            else { panic!("widest bounds should return a result"); };
            if i == 1 || i == 4 {
                ONE - sc1
            } else {
                sc1
            }
        })
        .collect::<Vec<Score>>();

    let error = Score::from_num(0.003);

    assert!(
        error + results[0] > results[1] && results[0] - error < results[1],
        "inversion 1 failed: {} != {}",
        results[0],
        results[1]
    );

    let correct = results[2];
    for (i, &inverted) in results[3..].iter().enumerate() {
        assert!(
            error + correct > inverted && correct - error < inverted,
            "inversion {} failed: {} != {}",
            i + 2,
            correct,
            inverted
        );
    }
}
