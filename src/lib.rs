mod utils;

use std::ops::Index;
use js_sys::{JsString, Array};
use wasm_bindgen::{prelude::*};
use chess::{
    BoardBuilder, Square, Rank, File, ChessMove, Piece, Color, CastleRights,
    BitBoard, EMPTY, PROMOTION_PIECES, ALL_PIECES
};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct MyBoard {
    bb: BoardBuilder,
    dead_moves: u8,
    status: Status
}

#[derive(Copy, Clone)]
struct MySquare(Square);

#[derive(Copy, Clone)]
enum Status {
    InProgress,
    Win(Color),
    Draw
}

#[wasm_bindgen]
impl MyBoard {
    pub fn initial_board() -> MyBoard {
        utils::set_panic_hook();
        MyBoard {
            bb: BoardBuilder::default(),
            dead_moves: 0,
            status: Status::InProgress
        }
    }

    pub fn js_piece(&self, file: usize, rank: usize) -> Option<JsString> {
        let square = MySquare::new(file, rank);
        match self[square] {
            Some((p, c)) => Some(p.to_string(c).into()),
            _ => None
        }
    }

    pub fn js_piece_color(&self, file: usize, rank: usize) -> JsString {
        let square = MySquare::new(file, rank);
        match self[square] {
            Some((_, Color::White)) => "white".into(),
            Some((_, Color::Black)) => "black".into(),
            _ => "empty".into()
        }
    }

    pub fn js_moves_from(&self, file: usize, rank: usize) -> Array {
        let square = MySquare::new(file, rank);
        let moves = self.moves_from(square);
        let js_moves = Array::new();
        for m in moves {
            let js_move = Array::new();
            js_move.push(&m.get_dest().get_file().to_index().into());
            js_move.push(&m.get_dest().get_rank().to_index().into());
            js_move.push(&
                if let Some(p) = m.get_promotion() { Some(p.to_index()) }
                else { None }
                .into()
            );
            js_moves.push(&js_move);
        }
        js_moves
    }

    pub fn js_apply_move(&mut self,
        from_file: usize, from_rank: usize, to_file: usize, to_rank: usize,
        promotion: Option<usize>
    ) {
        let from = MySquare::new(from_file, from_rank);
        let to = MySquare::new(to_file, to_rank);
        let m = ChessMove::new(from.0, to.0, promotion.map(|i| ALL_PIECES[i]));
        self.apply_move(m);
    }

    pub fn js_side_to_move(&self) -> JsString {
        if self.bb.get_side_to_move().to_index() == 0 { "white".into() }
        else { "black".into() }
    }

    pub fn js_status(&self) -> JsString {
        self.status.into()
    }
}

impl From<Status> for JsString {
    fn from(r: Status) -> JsString {
        match r {
            Status::InProgress => "in progress".into(),
            Status::Win(Color::White) => "white".into(),
            Status::Win(Color::Black) => "black".into(),
            Status::Draw => "draw".into()
        }
    }
}

impl MyBoard {

    fn moves_from(&self, sq: MySquare) -> Vec<ChessMove> {

        if !matches!(self.status, Status::InProgress) { return Vec::new(); }
        if self[sq].is_none() { return Vec::new(); }
        let (piece, color) = self[sq].unwrap();

        if color != self.bb.get_side_to_move() { return Vec::new(); }

        let mut moves = Vec::new();

        let not_self = !self.color_combined(color);
        let all = self.combined();

        // Add the normal moves
        for dest in (match piece {
            Piece::Pawn => chess::get_pawn_moves(sq.0, color, all),
            Piece::Knight => chess::get_knight_moves(sq.0),
            Piece::Bishop => chess::get_bishop_moves(sq.0, all),
            Piece::Rook => chess::get_rook_moves(sq.0, all),
            Piece::Queen =>
                chess::get_bishop_moves(sq.0, all)
                | chess::get_rook_moves(sq.0, all),
            Piece::King => chess::get_king_moves(sq.0)
        }) & not_self {
            moves.push(ChessMove::new(sq.0, dest, None));
        }
        
        // Add the castling moves
        if matches!(piece, Piece::King) {
            if
                self.bb.get_castle_rights(color).has_kingside()
                && all & CastleRights::Both.kingside_squares(color) == EMPTY
            {
                moves.push(ChessMove::new(
                    sq.0, MySquare::kingside_castle_square(color).0, None
                ));
            }
            if
                self.bb.get_castle_rights(color).has_queenside()
                && all & CastleRights::Both.queenside_squares(color) == EMPTY
            {
                moves.push(ChessMove::new(
                    sq.0, MySquare::queenside_castle_square(color).0, None
                ));
            }
        }

        // Transform backrank pawn moves to promotions
        if matches!(piece, Piece::Pawn) {
            moves = moves.into_iter().map(|m| {
                if m.get_dest().get_rank() == color.to_their_backrank() {
                    PROMOTION_PIECES.iter().map(|&p| {
                        ChessMove::new(m.get_source(), m.get_dest(), Some(p))
                    }).collect()
                } else { vec![m] }
            }).collect::<Vec<Vec<ChessMove>>>().concat();
        }
        
        moves
    }

    fn apply_move(&mut self, m: ChessMove) {
        assert!(self.moves_from(MySquare(m.get_source())).contains(&m));

        let (p, c) = self.bb[m.get_source()].unwrap();

        // Adjust the castling rights
        let opp = if c == Color::White { Color::Black } else { Color::White };
        // Remove castling rights based on piece moved
        self.bb.castle_rights(
            c,
            self.bb.get_castle_rights(c).remove(
                CastleRights::square_to_castle_rights(c, m.get_source())
            )
        );
        // Remove opponent castling rights based on piece taken
        self.bb.castle_rights(
            opp,
            self.bb.get_castle_rights(opp).remove(
                CastleRights::square_to_castle_rights(opp, m.get_dest())
            )
        );

        // Check if the king was taken for a win
        if  matches!(self.bb[m.get_dest()], Some((Piece::King, _))) {
            self.status = Status::Win(c);
        }

        // Apply the move
        self.bb.piece(m.get_dest(), p, c);
        self.bb.clear_square(m.get_source());

        // Promote
        if let Some(p) = m.get_promotion() {
            self.bb.piece(m.get_dest(), p, c);
        }

        // Switch turns
        self.bb.side_to_move(opp);

        // Detect 50 non-pawn non-capture moves for a draw
        if
            matches!(self.bb[m.get_dest()], Some(_))
            || matches!(p, Piece::Pawn)
        {
            self.dead_moves = 0;
        } else {
            self.dead_moves += 1;
            assert!(self.dead_moves <= 50);
            if self.dead_moves == 50 {
                self.status = Status::Draw;
            }
        }

    }

    // TODO: restructure BitBoard usage for efficiency
    fn color_combined(&self, c: Color) -> BitBoard {
        let mut bb = EMPTY;
        for sq in MySquare::all_squares() {
            match self[sq] {
                Some((_, color)) if color == c =>
                    bb |= BitBoard::from_square(sq.0),
                _ => {}
            }
        }
        bb
    }

    fn combined(&self) -> BitBoard {
        self.color_combined(Color::White) | self.color_combined(Color::Black)
    }

}

impl MySquare {
    fn new(file: usize, rank: usize) -> MySquare {
        MySquare(Square::make_square(
            Rank::from_index(rank),
            File::from_index(file)
        ))
    }
    fn all_squares() -> [MySquare; 64] {
        chess::ALL_SQUARES.map(|s| MySquare(s))
    }
    fn kingside_castle_square(color: Color) -> MySquare {
        match color {
            Color::White => MySquare(Square::make_square(Rank::First, File::G)),
            Color::Black => MySquare(Square::make_square(Rank::Eighth, File::G))
        }
    }
    fn queenside_castle_square(color: Color) -> MySquare {
        match color {
            Color::White => MySquare(Square::make_square(Rank::First, File::C)),
            Color::Black => MySquare(Square::make_square(Rank::Eighth, File::C))
        }
    }
}

impl Index<MySquare> for MyBoard {
    type Output = Option<(Piece, Color)>;

    fn index(&self, sq: MySquare) -> &Self::Output {
        &self.bb[sq.0]
    }
}