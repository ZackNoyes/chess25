use std::{
    sync::{Arc, Mutex},
    thread,
};

use chess::{Color, ALL_COLORS};
use rand::{thread_rng, Rng};
use random_chess::{
    bonus_chance, AlphaBeta, Engine, FeatureEval, Features, Logger, MyBoard, ProportionCount,
    StaticEvaluator, Status, Weights,
};

const LOG_LEVEL: u8 = 1;

fn main() { _run_concurrent_matches(); }

fn _feature_testing() {
    let weights1 = Weights {
        pieces: [[1.0, 3.0, 3.0, 5.0, 9.0, 0.0], [
            -1.0, -3.0, -3.0, -5.0, -9.0, 0.0,
        ]],
        king_danger: [-0.5, 0.5],
        pawn_advancement: [0.5, -0.5],
        side_to_move: 3.0,
    };

    let mut white = AlphaBeta::new(
        ProportionCount::default(),
        3,
        true,
        false,
        LOG_LEVEL,
        100000,
    );
    let mut black = AlphaBeta::new(
        ProportionCount::default(),
        3,
        true,
        false,
        LOG_LEVEL,
        100000,
    );

    let mut boards = Vec::new();

    for _ in 0..10 {
        let (_, new_boards) = _run_single_match(&mut white, &mut black);
        boards.extend(new_boards);
    }

    let mut lookahead = AlphaBeta::new(
        ProportionCount::default(),
        3,
        false,
        false,
        LOG_LEVEL,
        100000,
    );
    let mut new_lookahead = AlphaBeta::new(
        FeatureEval::new(weights1, 22.0),
        3,
        false,
        false,
        LOG_LEVEL,
        100000,
    );
    let static_eval = ProportionCount::default();
    let new_static_eval = FeatureEval::new(weights1, 14.0);

    let mut total_error = 0.0;
    let mut total_error2 = 0.0;
    let mut total_error3 = 0.0;

    let mut magnitude_error = 0.0;
    let mut magnitude_error2 = 0.0;

    for board in boards.iter() {
        let s_e: f32 = static_eval.evaluate(board).to_num();
        let s2_e: f32 = new_static_eval.evaluate(board).to_num();
        let l_e: f32 = lookahead.evaluate(board).to_num();
        let l2_e: f32 = new_lookahead.evaluate(board).to_num();
        println!("{}", board);
        println!("Static eval: {}", s_e);
        println!("Features eval: {}", s2_e);
        println!("Lookahead eval: {}", l_e);
        println!("Features lookahead eval: {}", l2_e);
        if board.get_status().is_in_progress() {
            println!("Features: {:?}", Features::from_board(board));
        }
        println!();
        total_error += (s_e - l_e).abs();
        total_error2 += (s2_e - l_e).abs();
        total_error3 += (s2_e - l2_e).abs();
        magnitude_error += (s2_e - 0.5).abs() - (l_e - 0.5).abs();
        magnitude_error2 += (s2_e - 0.5).abs() - (l2_e - 0.5).abs();
    }

    println!(
        "Average error propcount {}",
        total_error / boards.len() as f32
    );
    println!(
        "Average error features {}",
        total_error2 / boards.len() as f32
    );
    println!(
        "Average self-error features {}",
        total_error3 / boards.len() as f32
    );
    println!(
        "Magnitude error features {}",
        magnitude_error / boards.len() as f32
    );
    println!(
        "Magnitude self-error features {}",
        magnitude_error2 / boards.len() as f32
    );
}

fn _run_single_match(
    white_player: &mut dyn Engine, black_player: &mut dyn Engine,
) -> (Status, Vec<MyBoard>) {
    let mut rng = thread_rng();
    let mut board = MyBoard::initial_board(ALL_COLORS[rng.gen_range(0..=1)]);
    let mut boards = vec![board];

    loop {
        if !board.get_status().is_in_progress() {
            break;
        }

        let mv = if board.get_side_to_move() == Color::White {
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
    let mut white = AlphaBeta::new(
        ProportionCount::default(),
        3,
        false,
        false,
        LOG_LEVEL,
        100000,
    );
    let mut black = AlphaBeta::new(
        ProportionCount::default(),
        3,
        false,
        false,
        LOG_LEVEL,
        100000,
    );

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
                pieces: [[1.0, 3.0, 3.0, 5.0, 9.0, 0.0], [
                    -1.0, -3.0, -3.0, -5.0, -9.0, 0.0,
                ]],
                king_danger: [-0.5, 0.5],
                pawn_advancement: [0.5, -0.5],
                side_to_move: 3.0,
            };
            let weights2 = Weights {
                pieces: [[1.0, 3.0, 3.0, 5.0, 9.0, 0.0], [
                    -1.0, -3.0, -3.0, -5.0, -9.0, 0.0,
                ]],
                king_danger: [-0.5, 0.5],
                pawn_advancement: [0.5, -0.5],
                side_to_move: 3.0,
            };

            let mut logger = Logger::new(LOG_LEVEL);

            let mut white = AlphaBeta::new(
                FeatureEval::new(weights1, 15.0),
                10,
                true,
                false,
                LOG_LEVEL,
                100,
            );
            let mut black = AlphaBeta::new(
                FeatureEval::new(weights2, 15.0),
                10,
                true,
                false,
                LOG_LEVEL,
                100,
            );
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
                    _ => unreachable!(),
                }
                println!(
                    "{}: White wins: {}, Black wins: {}, Draws: {}",
                    t,
                    white_wins.lock().unwrap(),
                    black_wins.lock().unwrap(),
                    draws.lock().unwrap()
                )
            }
        }));
    }

    for handle in thread_handles {
        handle.join().unwrap();
    }
}
