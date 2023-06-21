
use std::ops::Index;
use chess::{
    BoardBuilder, Square, Rank, File, ChessMove, Piece, Color, CastleRights,
    BitBoard, EMPTY, PROMOTION_PIECES
};

#[derive(Copy, Clone)]
pub struct MyBoard {
    board: BoardBuilder,
    dead_moves: u8,
    status: Status,
    awaiting_bonus: bool,
    white_pieces: BitBoard,
    black_pieces: BitBoard,
}

#[derive(Copy, Clone)]
pub struct MySquare(pub Square);

#[derive(Copy, Clone)]
pub enum Status {
    InProgress,
    Win(Color),
    Draw
}

impl MyBoard {

    pub fn get_board(&self) -> &BoardBuilder {
        &self.board
    }

    pub fn get_status(&self) -> Status {
        self.status
    }

    pub fn initial_board(starting_color: Color) -> MyBoard {
        let board = BoardBuilder::default();
        let mut white_pieces = EMPTY;
        let mut black_pieces = EMPTY;
        for sq in chess::ALL_SQUARES {
            match board[sq] {
                Some((_, Color::White)) => white_pieces |= BitBoard::from_square(sq),
                Some((_, Color::Black)) => black_pieces |= BitBoard::from_square(sq),
                _ => (),
            }
        }
        let mut board = MyBoard {
            board,
            dead_moves: 0,
            status: Status::InProgress,
            awaiting_bonus: false,
            white_pieces,
            black_pieces,
        };
        board.board.side_to_move(starting_color);
        board
    }

    pub fn moves_from(&self, sq: MySquare) -> Vec<ChessMove> {
        assert!(!self.awaiting_bonus, "Tried to request move from board awaiting bonus");
        
        if !matches!(self.status, Status::InProgress) { return Vec::new(); }
        if self[sq].is_none() { return Vec::new(); }
        let (piece, color) = self[sq].unwrap();

        if color != self.board.get_side_to_move() { return Vec::new(); }

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
                self.board.get_castle_rights(color).has_kingside()
                && all & CastleRights::Both.kingside_squares(color) == EMPTY
            {
                moves.push(ChessMove::new(
                    sq.0, MySquare::kingside_castle_square(color).0, None
                ));
            }
            if
                self.board.get_castle_rights(color).has_queenside()
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

    pub fn apply_move(&mut self, m: ChessMove) {
        assert!(self.moves_from(MySquare(m.get_source())).contains(&m));
        assert!(!self.awaiting_bonus); self.awaiting_bonus = true;

        let (p, c) = self.board[m.get_source()].unwrap();

        // Adjust the castling rights
        let opp = if c == Color::White { Color::Black } else { Color::White };
        // Remove castling rights based on piece moved
        self.board.castle_rights(
            c,
            self.board.get_castle_rights(c).remove(
                CastleRights::square_to_castle_rights(c, m.get_source())
            )
        );
        // Remove opponent castling rights based on piece taken
        self.board.castle_rights(
            opp,
            self.board.get_castle_rights(opp).remove(
                CastleRights::square_to_castle_rights(opp, m.get_dest())
            )
        );

        // Check if the king was taken for a win
        if  matches!(self.board[m.get_dest()], Some((Piece::King, _))) {
            self.status = Status::Win(c);
        }

        // Detect 50 non-pawn non-capture moves for a draw
        if
            matches!(self.board[m.get_dest()], Some(_))
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

        // Apply the move
        self.board.piece(m.get_dest(), p, c);
        self.board.clear_square(m.get_source());
        self.update_bitboards(c, m.get_source(), m.get_dest());

        // Handle castling
        if matches!(p, Piece::King) && matches!(m.get_source().get_file(), File::E) {
            let mut src = None; let mut dst = None;
            if m.get_dest().get_file() == File::G {
                src = Some(if c == Color::White { Square::H1 } else { Square::H8 });
                dst = Some(if c == Color::White { Square::F1 } else { Square::F8 });
            } else if m.get_dest().get_file() == File::C {
                src = Some(if c == Color::White { Square::A1 } else { Square::A8 });
                dst = Some(if c == Color::White { Square::D1 } else { Square::D8 });
            }
            if let (Some(src), Some(dst)) = (src, dst) {
                self.board.piece(dst, Piece::Rook, c);
                self.board.clear_square(src);
                self.update_bitboards(c, src, dst);
            }
        }

        // Promote
        if let Some(p) = m.get_promotion() {
            self.board.piece(m.get_dest(), p, c);
        }

        // Switch turns
        self.board.side_to_move(opp);

    }

    /// Updates the internal bitboards to reflect a move by the color `c`
    /// from `src` to `dst`.
    fn update_bitboards(&mut self, c: Color, src: Square, dst: Square) {
        match c {
            Color::White => {
                self.white_pieces &= !BitBoard::from_square(src);
                self.white_pieces |= BitBoard::from_square(dst);
                self.black_pieces &= !BitBoard::from_square(dst);
            }
            Color::Black => {
                self.black_pieces &= !BitBoard::from_square(src);
                self.black_pieces |= BitBoard::from_square(dst);
                self.white_pieces &= !BitBoard::from_square(dst);
            }
        }
    }

    /// Applies the bonus move but doesn't check for a draw
    pub fn apply_bonus_unchecked(&mut self, is_bonus: bool) {
        assert!(self.awaiting_bonus); self.awaiting_bonus = false;

        if is_bonus {
            let opp = if self.board.get_side_to_move() == Color::White {
                Color::Black
            } else { Color::White };
            self.board.side_to_move(opp);
        }
    }

    pub fn apply_bonus(&mut self, is_bonus: bool) {
        self.apply_bonus_unchecked(is_bonus);

        // Detect no moves draw
        if self.all_moves().is_empty() && matches!(self.status, Status::InProgress) {
            self.status = Status::Draw;
        }
    }

    pub fn all_moves(&self) -> Vec<ChessMove> {
        MySquare::all_squares().iter().map(|&sq| self.moves_from(sq))
            .collect::<Vec<Vec<ChessMove>>>().concat()
    }

    fn color_combined(&self, c: Color) -> BitBoard {
        match c {
            Color::White => self.white_pieces,
            Color::Black => self.black_pieces,
        }
    }

    fn combined(&self) -> BitBoard {
        self.color_combined(Color::White) | self.color_combined(Color::Black)
    }

}

impl MySquare {
    pub fn new(file: usize, rank: usize) -> MySquare {
        MySquare(Square::make_square(
            Rank::from_index(rank),
            File::from_index(file)
        ))
    }
    pub fn all_squares() -> [MySquare; 64] {
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
        &self.board[sq.0]
    }
}