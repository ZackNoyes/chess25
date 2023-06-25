use chess::{ALL_COLORS, Color};
use random_chess::{Engine, MyBoard, Status, bonus_chance, ProportionCount};
use random_chess::AlphaBeta;
use rand::{thread_rng, Rng};

fn main() {
    
    let mut white = AlphaBeta::new(ProportionCount::default(), 3, false, 10);
    let mut black = AlphaBeta::new(ProportionCount::default(), 3, false, 10);

    let (res, moves) = run_single_match(&mut white, &mut black);

    println!("Result: {} in {} moves", res, moves);

}

fn run_single_match(white_player: &mut dyn Engine, black_player: &mut dyn Engine)
    -> (Status, usize)
{
    let mut rng = thread_rng();
    let mut board = MyBoard::initial_board(ALL_COLORS[rng.gen_range(0..=1)]);
    let mut moves = 0;
    loop {
        if !matches!(board.get_status(), Status::InProgress) { break; }

        let mv = if matches!(board.get_side_to_move(), Color::White) {
            white_player.get_move(&board)
        } else {
            black_player.get_move(&board)
        };

        board.apply_move(mv);
        board.apply_bonus(rng.gen_bool(bonus_chance().into()));
        moves += 1;
    }
    (board.get_status(), moves)
}