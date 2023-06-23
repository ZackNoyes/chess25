
use std::ops::Index;
use chess::{
    BoardBuilder, Square, Rank, File, ChessMove, Piece, Color, CastleRights,
    BitBoard, EMPTY, PROMOTION_PIECES, ALL_SQUARES,
};
use crate::zobrist::Zobrist;

#[derive(Copy, Clone)]
pub struct MyBoard {
    pieces: [Option<(Piece, Color)>; 64],
    side_to_move: Color,
    castle_rights: [CastleRights; 2],
    dead_moves: u8,
    status: Status,
    awaiting_bonus: bool, // TODO: refactor into side_to_move
    white_pieces: BitBoard,
    black_pieces: BitBoard,
    zobrist_hash: u64,
}

#[derive(Copy, Clone)]
pub enum Status {
    InProgress,
    Win(Color),
    Draw
}

impl MyBoard {

    pub fn get_side_to_move(&self) -> Color { self.side_to_move }
    pub fn get_castle_rights(&self, color: Color) -> CastleRights {
        match color {
            Color::White => self.castle_rights[0],
            Color::Black => self.castle_rights[1],
        }
    }
    pub fn get_dead_moves(&self) -> u8 { self.dead_moves }
    pub fn get_status(&self) -> Status { self.status }
    pub fn get_white_pieces(&self) -> BitBoard { self.white_pieces }
    pub fn get_black_pieces(&self) -> BitBoard { self.black_pieces }
    pub fn get_zobrist_hash(&self) -> u64 { self.zobrist_hash }

    /// Sets the castle rights, updating the zobrist hash
    fn set_castle_rights(&mut self, color: Color, rights: CastleRights) {
        self.zobrist_hash ^= Zobrist::castles(self.get_castle_rights(color), color);
        self.zobrist_hash ^= Zobrist::castles(rights, color);
        match color {
            Color::White => self.castle_rights[0] = rights,
            Color::Black => self.castle_rights[1] = rights,
        }
    }

    pub fn initial_board(starting_color: Color) -> MyBoard {
        let board = BoardBuilder::default();
        let mut pieces = [None; 64];
        let mut white_pieces = EMPTY;
        let mut black_pieces = EMPTY;
        let mut zobrist_hash = 0;
        for sq in ALL_SQUARES {
            if let Some((piece, color)) = board[sq] {
                pieces[sq.to_index()] = Some((piece, color));
                match color {
                    Color::White => white_pieces |= BitBoard::from_square(sq),
                    Color::Black => black_pieces |= BitBoard::from_square(sq),
                }
                zobrist_hash ^= Zobrist::piece(piece, sq, color);
            }
        }
        zobrist_hash ^= Zobrist::castles(CastleRights::Both, Color::White);
        zobrist_hash ^= Zobrist::castles(CastleRights::Both, Color::Black);
        if matches!(starting_color, Color::Black) {
            zobrist_hash ^= Zobrist::color();
        }
        let board = MyBoard {
            pieces,
            side_to_move: starting_color,
            castle_rights: [CastleRights::Both, CastleRights::Both],
            dead_moves: 0,
            status: Status::InProgress,
            awaiting_bonus: false,
            white_pieces,
            black_pieces,
            zobrist_hash,
        };
        board
    }

    pub fn moves_from(&self, sq: Square) -> Vec<ChessMove> {
        assert!(!self.awaiting_bonus, "Tried to request move from board awaiting bonus");
        
        if !matches!(self.status, Status::InProgress) { return Vec::new(); }
        if self[sq].is_none() { return Vec::new(); }
        let (piece, color) = self[sq].unwrap();

        if color != self.side_to_move { return Vec::new(); }

        let mut moves = Vec::new();

        let not_self = !self.color_combined(color);
        let all = self.combined();

        // Add the normal moves
        for dest in (match piece {
            Piece::Pawn => chess::get_pawn_moves(sq, color, all),
            Piece::Knight => chess::get_knight_moves(sq),
            Piece::Bishop => chess::get_bishop_moves(sq, all),
            Piece::Rook => chess::get_rook_moves(sq, all),
            Piece::Queen =>
                chess::get_bishop_moves(sq, all)
                | chess::get_rook_moves(sq, all),
            Piece::King => chess::get_king_moves(sq)
        }) & not_self {
            moves.push(ChessMove::new(sq, dest, None));
        }
        
        // Add the castling moves
        if matches!(piece, Piece::King) {
            if
                self.get_castle_rights(color).has_kingside()
                && all & CastleRights::Both.kingside_squares(color) == EMPTY
            {
                moves.push(ChessMove::new(
                    sq, kingside_castle_square(color), None
                ));
            }
            if
                self.get_castle_rights(color).has_queenside()
                && all & CastleRights::Both.queenside_squares(color) == EMPTY
            {
                moves.push(ChessMove::new(
                    sq, queenside_castle_square(color), None
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
        assert!(self.moves_from(m.get_source()).contains(&m));
        assert!(!self.awaiting_bonus); self.awaiting_bonus = true;

        let (p, c) = self[m.get_source()].expect("No piece at source");

        // Adjust the castling rights
        // Remove castling rights based on piece moved
        self.set_castle_rights(
            c,
            self.get_castle_rights(c).remove(
                CastleRights::square_to_castle_rights(c, m.get_source())
            )
        );
        // Remove opponent castling rights based on piece taken
        self.set_castle_rights(
            !c,
            self.get_castle_rights(!c).remove(
                CastleRights::square_to_castle_rights(!c, m.get_dest())
            )
        );

        // Check if the king was taken for a win
        if  matches!(self[m.get_dest()], Some((Piece::King, _))) {
            self.status = Status::Win(c);
        }

        // Detect 50 non-pawn non-capture moves for a draw
        if
            matches!(self[m.get_dest()], Some(_))
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
        self.set_piece(m.get_dest(), Some((p, c)));
        self.set_piece(m.get_source(), None);

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
                self.set_piece(dst, Some((Piece::Rook, c)));
                self.set_piece(src, None);
            }
        }

        // Promote
        if let Some(p) = m.get_promotion() {
            self.set_piece(m.get_dest(), Some((p, c)));
        }

        // Switch turns
        self.switch_side_to_move();

    }

    /// Updates the piece at a particular square. Also updates the bitboards
    /// and zobrist hash.
    fn set_piece(&mut self, sq: Square, piece: Option<(Piece, Color)>) {
        if let Some((p, c)) = self[sq] {
            if c == Color::White {
                self.white_pieces &= !BitBoard::from_square(sq);
            } else {
                self.black_pieces &= !BitBoard::from_square(sq);
            }
            self.zobrist_hash ^= Zobrist::piece(p, sq, c);
        }
        if let Some((p, c)) = piece {
            if c == Color::White {
                self.white_pieces |= BitBoard::from_square(sq);
            } else {
                self.black_pieces |= BitBoard::from_square(sq);
            }
            self.zobrist_hash ^= Zobrist::piece(p, sq, c);
        }
        self.pieces[sq.to_index()] = piece;
    }

    /// Switches the side to move and updates the zobrist hash.
    fn switch_side_to_move(&mut self) {
        self.zobrist_hash ^= Zobrist::color();
        self.side_to_move = !self.side_to_move;
    }

    /// Applies the bonus move but doesn't check for a draw
    pub fn apply_bonus_unchecked(&mut self, is_bonus: bool) {
        assert!(self.awaiting_bonus); self.awaiting_bonus = false;
        if is_bonus { self.switch_side_to_move() }
    }

    pub fn apply_bonus(&mut self, is_bonus: bool) {
        self.apply_bonus_unchecked(is_bonus);

        // Detect no moves draw
        if self.all_moves().is_empty() && matches!(self.status, Status::InProgress) {
            self.status = Status::Draw;
        }
    }

    pub fn all_moves(&self) -> Vec<ChessMove> {
        ALL_SQUARES.iter().map(|&sq| self.moves_from(sq))
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

fn kingside_castle_square(color: Color) -> Square {
    match color {
        Color::White => Square::make_square(Rank::First, File::G),
        Color::Black => Square::make_square(Rank::Eighth, File::G)
    }
}

fn queenside_castle_square(color: Color) -> Square {
    match color {
        Color::White => Square::make_square(Rank::First, File::C),
        Color::Black => Square::make_square(Rank::Eighth, File::C)
    }
}

impl Index<Square> for MyBoard {
    type Output = Option<(Piece, Color)>;
    fn index(&self, sq: Square) -> &Self::Output {
        &self.pieces[sq.to_index()]
    }
}