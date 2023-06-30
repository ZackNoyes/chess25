use std::io::Write;

use chess::{
    ChessMove,
    Color::{self, Black, White},
    File,
    Piece::Pawn,
    Rank, Square, ALL_PIECES,
};
use clap::{Parser, ValueEnum};
use random_chess::{AlphaBeta, Engine, FeatureEval, Status, Weights};

const INSTRUCTIONS: &str = "\
    Please enter your move as 5 space-separated integers:\n    \
      <from_file> <from_rank> <to_file> <to_rank> <promotion>\n      \
      For files: 0 means A, 1 means B, ..., 7 means H.\n      \
      For ranks: 0 means 1st, 1 means 2nd, ..., 7 means 8th.\n      \
      For promotion: 1 means knight, 2 means bishop, 3 means rook, 4 means queen.\n  \
      Example: \n    \
        - the opening move 1. e4 would be entered as \"4 1 4 3 0\".\n    \
        - a promotion to a queen pushing white's e-pawn would be \"4 7 4 8 4\".\n\
";

/// Arguments to the engine
#[derive(Parser, Debug)]
#[command(
    name = "Random Chess - Engine",
    about = "Simple binary for running my engine for my game Random Chess." // TODO: Instructions
)]
struct Cli {
    /// The color the engine should play as
    #[arg(short, long, default_value = "white")]
    engine_color: ArgColor,
    /// The color who should start the game
    #[arg(short, long, default_value = "white")]
    starting_color: ArgColor,
    /// The timeout for the engine in milliseconds
    #[arg(short, long, default_value = "4000")]
    timeout: u64,
    /// Whether to prevent the game board, human-readable moves, and prompts
    /// from being printed
    #[arg(short, long)]
    quiet: bool,
}

#[derive(ValueEnum, Copy, Clone, Debug)]
enum ArgColor {
    White,
    Black,
}

impl ArgColor {
    fn to_color(&self) -> Color {
        match self {
            ArgColor::White => White,
            ArgColor::Black => Black,
        }
    }
}

fn main() {
    let cli = Cli::parse();

    let weights = Weights {
        pieces: [[1.0, 3.0, 3.0, 5.0, 9.0, 0.0], [
            -1.0, -3.0, -3.0, -5.0, -9.0, 0.0,
        ]],
        king_danger: [-0.5, 0.5],
        pawn_advancement: [0.5, -0.5],
        side_to_move: 3.0,
    };

    let mut engine = AlphaBeta::new(
        FeatureEval::new(weights, 15.0),
        10,
        true,
        true,
        0,
        cli.timeout,
    );

    let mut board = random_chess::MyBoard::initial_board(cli.starting_color.to_color());

    while board.get_status().is_in_progress() {
        if board.get_side_to_move() == cli.engine_color.to_color() {
            let mv = engine.get_move(&board);
            if !cli.quiet {
                println!(
                    "Engine played: {} {} {} {} {} [{} -> {}]",
                    mv.get_source().get_file().to_index(),
                    mv.get_source().get_rank().to_index(),
                    mv.get_dest().get_file().to_index(),
                    mv.get_dest().get_rank().to_index(),
                    mv.get_promotion().unwrap_or(Pawn).to_index(),
                    mv.get_source(),
                    mv.get_dest(),
                );
            } else {
                println!(
                    "{} {} {} {} {}",
                    mv.get_source().get_file().to_index(),
                    mv.get_source().get_rank().to_index(),
                    mv.get_dest().get_file().to_index(),
                    mv.get_dest().get_rank().to_index(),
                    mv.get_promotion().unwrap_or(Pawn).to_index(),
                );
            }
            board.apply_move(mv);
        } else {
            if !cli.quiet {
                println!("{}", board);
                print!("Enter your move: ");
                std::io::stdout().flush().unwrap();
            }
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let nums = input
                .trim()
                .split(' ')
                .map(|s| s.parse::<usize>().ok().filter(|&n| n < 8))
                .collect::<Vec<_>>();
            if let Some(p) = nums.get(4) {
                p.filter(|&n| n < 5);
            }
            let nums = if nums.len() == 5 && nums.iter().all(|n| n.is_some()) {
                Some(nums.into_iter().map(|n| n.unwrap()).collect::<Vec<_>>())
            } else {
                None
            };
            let Some(nums) = nums else {
                println!("Invalid input.");
                println!("{}", INSTRUCTIONS);
                continue;
            };
            let mv = ChessMove::new(
                Square::make_square(Rank::from_index(nums[1]), File::from_index(nums[0])),
                Square::make_square(Rank::from_index(nums[3]), File::from_index(nums[2])),
                if nums[4] == 0 {
                    None
                } else {
                    Some(ALL_PIECES[nums[4]])
                },
            );
            if !board.moves_from(mv.get_source()).contains(&mv) {
                println!("Illegal move.");
                println!("{}", INSTRUCTIONS);
                continue;
            }
            board.apply_move(mv);
        }
        if !board.get_status().is_in_progress() {
            break;
        }
        let bonus = loop {
            if !cli.quiet {
                print!("\"bonus\" or \"no_bonus\": ");
                std::io::stdout().flush().unwrap();
            }
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let bonus = match input.trim() {
                "bonus" => true,
                "no_bonus" => false,
                _ => {
                    println!("Please enter \"bonus\" or \"no_bonus\".");
                    continue;
                }
            };
            break bonus;
        };
        board.apply_bonus(bonus);
    }
    match board.get_status() {
        Status::Win(White) => {
            println!("white wins");
        }
        Status::Win(Black) => {
            println!("black wins");
        }
        Status::Draw => {
            println!("draw");
        }
        _ => unreachable!(),
    }
}
