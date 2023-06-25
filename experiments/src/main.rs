use chess::{ALL_COLORS, Color};
use random_chess::{Engine, MyBoard, Status, Logger, bonus_chance};
use random_chess::{AlphaBeta, ProportionCount};
use rand::{thread_rng, Rng};

use std::sync::{Arc, Mutex};
use std::thread;

const LOG_LEVEL: u8 = 1;

fn main() {

    _run_concurrent_matches();

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

fn _bench_single_match() {
    let mut logger = Logger::new(LOG_LEVEL);
    let mut white = AlphaBeta::new(ProportionCount::default(), 3, false, LOG_LEVEL);
    let mut black = AlphaBeta::new(ProportionCount::default(), 3, false, LOG_LEVEL);

    logger.time_start(1, "single match time");
    let (res, moves) = run_single_match(&mut white, &mut black);
    println!("Result: {} in {} moves", res, moves);
    logger.time_end(1, "single match time");
}

fn _run_concurrent_matches() {
    let white_wins = Arc::new(Mutex::new(0));
    let black_wins = Arc::new(Mutex::new(0));
    let draws = Arc::new(Mutex::new(0));

    let mut thread_handles = Vec::new();

    for t in 1..=5 {
        let white_wins = Arc::clone(&white_wins);
        let black_wins = Arc::clone(&black_wins);
        let draws = Arc::clone(&draws);
        thread_handles.push(thread::spawn(move || {
            let mut logger = Logger::new(LOG_LEVEL);
            // TODO: find the bug that leads to white winning more often
            let mut white = AlphaBeta::new(ProportionCount::default(), 1, false, LOG_LEVEL);
            let mut black = AlphaBeta::new(ProportionCount::default(), 1, false, LOG_LEVEL);
            for _ in 1..=100 {
                // println!("{}: Match {}", t, i);
                logger.time_start(1, "single match time");
                let (res, _) = run_single_match(&mut white, &mut black);
                // println!("{}: Result: {} in {} moves", t, res, moves);
                logger.time_end(1, "single match time");
                match res {
                    Status::Win(Color::White) => *white_wins.lock().unwrap() += 1,
                    Status::Win(Color::Black) => *black_wins.lock().unwrap() += 1,
                    Status::Draw => *draws.lock().unwrap() += 1,
                    _ => unreachable!()
                }
                println!("{}: White wins: {}, Black wins: {}, Draws: {}", t,
                    white_wins.lock().unwrap(), black_wins.lock().unwrap(),
                    draws.lock().unwrap())
            }
        }));
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
}