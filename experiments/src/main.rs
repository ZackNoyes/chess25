use chess::{ALL_COLORS, Color};
use random_chess::{Engine, MyBoard, Status, Logger, bonus_chance};
use random_chess::{AlphaBeta, StaticEvaluator, ProportionCount, FeatureEval, Weights};
use random_chess::Features;
use rand::{thread_rng, Rng};

use std::sync::{Arc, Mutex};
use std::thread;

const LOG_LEVEL: u8 = 1;

fn main() {
    _run_concurrent_matches();
}

fn _feature_testing() {
    let weights1 = Weights {
        pieces: [[1.0, 3.0, 3.0, 5.0, 9.0, 0.0], [-1.0, -3.0, -3.0, -5.0, -9.0, 0.0]],
        mobility: [0.5, -0.5],
        king_danger: [-2.0, 2.0],
        pawn_advancement: [1.0, -1.0],
        side_to_move: 3.0,
    };

    let mut white = AlphaBeta::new(ProportionCount::default(), 3, true, LOG_LEVEL);
    let mut black = AlphaBeta::new(ProportionCount::default(), 3, true, LOG_LEVEL);

    let mut boards = Vec::new();

    for _ in 0..10 {
        let (_, new_boards) = _run_single_match(&mut white, &mut black);
        boards.extend(new_boards);
    }

    let mut lookahead = AlphaBeta::new(ProportionCount::default(), 3, false, LOG_LEVEL);
    let static_eval = ProportionCount::default();
    let new_static_eval = FeatureEval::new(weights1, 22.0);

    let mut total_error = 0.0;
    let mut total_error2 = 0.0;

    for board in boards.iter() {
        let s_e: f32 = static_eval.evaluate(board).to_num();
        let s2_e: f32 = new_static_eval.evaluate(board).to_num();
        let l_e: f32 = lookahead.evaluate(board).to_num();
        println!("{}", board);
        println!("Static eval: {}", s_e);
        println!("Features eval: {}", s2_e);
        println!("Lookahead eval: {}", l_e);
        if matches!(board.get_status(), Status::InProgress) {
            println!("Features: {:?}", Features::from_board(board));
        }
        println!();
        total_error += (s_e - l_e).abs();
        total_error2 += (s2_e - l_e).abs();
    }

    println!("Average error propcount {}", total_error / boards.len() as f32);
    println!("Average error features {}", total_error2 / boards.len() as f32);
}

fn _run_single_match(white_player: &mut dyn Engine, black_player: &mut dyn Engine)
    -> (Status, Vec<MyBoard>)
{
    let mut rng = thread_rng();
    let mut board = MyBoard::initial_board(ALL_COLORS[rng.gen_range(0..=1)]);
    let mut boards = vec![board];

    loop {
        if !matches!(board.get_status(), Status::InProgress) { break; }

        let mv = if matches!(board.get_side_to_move(), Color::White) {
            white_player.get_move(&board)
        } else {
            black_player.get_move(&board)
        };

        board.apply_move(mv);
        board.apply_bonus(rng.gen_bool(bonus_chance().into()));
        boards.push(board);
    }
    (board.get_status(), boards)
}

fn _bench_single_match() {
    let mut logger = Logger::new(LOG_LEVEL);
    let mut white = AlphaBeta::new(ProportionCount::default(), 3, false, LOG_LEVEL);
    let mut black = AlphaBeta::new(ProportionCount::default(), 3, false, LOG_LEVEL);

    logger.time_start(1, "single match time");
    let (res, boards) = _run_single_match(&mut white, &mut black);
    println!("Result: {} in {} moves", res, boards.len());
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
            
            let weights1 = Weights {
                pieces: [[1.0, 3.0, 3.0, 5.0, 9.0, 0.0], [-1.0, -3.0, -3.0, -5.0, -9.0, 0.0]],
                mobility: [0.5, -0.5],
                king_danger: [-2.0, 2.0],
                pawn_advancement: [1.0, -1.0],
                side_to_move: 3.0,
            };

            let mut logger = Logger::new(LOG_LEVEL);
            
            let mut black = AlphaBeta::new(ProportionCount::default(), 2, false, LOG_LEVEL);
            let mut white = AlphaBeta::new(FeatureEval::new(weights1, 22.0), 2, false, LOG_LEVEL);
            for _ in 1..=200 {
                // println!("{}: Match {}", t, i);
                logger.time_start(1, "single match time");
                let (res, _) = _run_single_match(&mut white, &mut black);
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