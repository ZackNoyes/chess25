use chess::Color;

use crate::{engine::proportion_count};

use super::*;

#[test]
#[ignore]
fn test_self_game() {

    let mut white = AlphaBeta::new(proportion_count::ProportionCount::default(), 2, false, 10);
    let mut black = AlphaBeta::new(proportion_count::ProportionCount::default(), 2, false, 10);

    let mut board = MyBoard::initial_board(Color::White);

    let mut moves = 0;

    loop {
        if !matches!(board.get_status(), Status::InProgress) { break; }

        let mv = if matches!(board.get_side_to_move(), Color::White) {
            white.get_move(&board)
        } else {
            black.get_move(&board)
        };

        board.apply_move(mv);
        board.apply_bonus(moves % 5 == 0);
        moves += 1;
        println!("{}", board);
        check_inversions(&board);
    }

}

fn check_inversions(board: &MyBoard) {

    let mut boards = [*board; 4];
    boards[1].invert_files();
    boards[2].invert_ranks_and_colors();
    boards[3].invert_files();
    boards[3].invert_ranks_and_colors();

    let results = boards.iter().enumerate().map(|(i, b)| {
        let mut engine = AlphaBeta::new(proportion_count::ProportionCount::default(), 3, false, 0);
        let Result(sc1, _) = engine.get_scored_best_move(&b, Bounds::widest(), 3)
            else { panic!("widest bounds should return a result"); };
        if i<2 { sc1 } else { ONE - sc1 }
    }).collect::<Vec<Score>>();

    let error = Score::from_num(0.001);

    let r1 = results[0];

    for (i, &r2) in results[1..].iter().enumerate() {
        assert!(
            error + r1 > r2 && r1 - error < r2,
            "inversion {} failed: {} != {}",
            i, r1, r2
        );
    }
}