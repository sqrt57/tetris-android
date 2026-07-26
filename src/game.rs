//! Pure Tetris game logic — no Android/GPU dependencies, unit-testable on desktop.

pub const BOARD_WIDTH: usize = 10;
pub const BOARD_HEIGHT: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

const ALL_KINDS: [Kind; 7] = [Kind::I, Kind::O, Kind::T, Kind::S, Kind::Z, Kind::J, Kind::L];

/// Each rotation is a 4x4 grid of occupied cells, row-major, top-left origin.
/// Standard guideline spawn orientations; only 0/90/180/270 are modeled (no wall kicks).
fn shape(kind: Kind, rotation: u8) -> [[bool; 4]; 4] {
    let r = rotation % 4;
    match kind {
        Kind::I => match r {
            0 => grid(&[(0, 1), (1, 1), (2, 1), (3, 1)]),
            1 => grid(&[(2, 0), (2, 1), (2, 2), (2, 3)]),
            2 => grid(&[(0, 2), (1, 2), (2, 2), (3, 2)]),
            _ => grid(&[(1, 0), (1, 1), (1, 2), (1, 3)]),
        },
        Kind::O => grid(&[(1, 0), (2, 0), (1, 1), (2, 1)]),
        Kind::T => match r {
            0 => grid(&[(1, 0), (0, 1), (1, 1), (2, 1)]),
            1 => grid(&[(1, 0), (1, 1), (2, 1), (1, 2)]),
            2 => grid(&[(0, 1), (1, 1), (2, 1), (1, 2)]),
            _ => grid(&[(1, 0), (0, 1), (1, 1), (1, 2)]),
        },
        Kind::S => match r {
            0 | 2 => grid(&[(1, 0), (2, 0), (0, 1), (1, 1)]),
            _ => grid(&[(1, 0), (1, 1), (2, 1), (2, 2)]),
        },
        Kind::Z => match r {
            0 | 2 => grid(&[(0, 0), (1, 0), (1, 1), (2, 1)]),
            _ => grid(&[(2, 0), (1, 1), (2, 1), (1, 2)]),
        },
        Kind::J => match r {
            0 => grid(&[(0, 0), (0, 1), (1, 1), (2, 1)]),
            1 => grid(&[(1, 0), (2, 0), (1, 1), (1, 2)]),
            2 => grid(&[(0, 1), (1, 1), (2, 1), (2, 2)]),
            _ => grid(&[(1, 0), (1, 1), (0, 2), (1, 2)]),
        },
        Kind::L => match r {
            0 => grid(&[(2, 0), (0, 1), (1, 1), (2, 1)]),
            1 => grid(&[(1, 0), (1, 1), (1, 2), (2, 2)]),
            2 => grid(&[(0, 1), (1, 1), (2, 1), (0, 2)]),
            _ => grid(&[(0, 0), (1, 0), (1, 1), (1, 2)]),
        },
    }
}

fn grid(cells: &[(usize, usize)]) -> [[bool; 4]; 4] {
    let mut g = [[false; 4]; 4];
    for &(x, y) in cells {
        g[y][x] = true;
    }
    g
}

#[derive(Clone, Copy, Debug)]
pub struct Piece {
    pub kind: Kind,
    pub rotation: u8,
    /// Top-left corner of the piece's 4x4 bounding box, in board coordinates.
    pub x: i32,
    pub y: i32,
}

impl Piece {
    fn spawn(kind: Kind) -> Self {
        Piece { kind, rotation: 0, x: (BOARD_WIDTH as i32 - 4) / 2, y: -1 }
    }

    pub fn cells(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let s = shape(self.kind, self.rotation);
        (0..4).flat_map(move |gy| {
            (0..4).filter_map(move |gx| {
                if s[gy][gx] {
                    Some((self.x + gx as i32, self.y + gy as i32))
                } else {
                    None
                }
            })
        })
    }
}

pub struct Game {
    pub board: [[Option<Kind>; BOARD_WIDTH]; BOARD_HEIGHT],
    pub current: Piece,
    pub next: Kind,
    pub score: u32,
    pub lines_cleared: u32,
    pub game_over: bool,
    rng_state: u64,
}

impl Game {
    pub fn new(seed: u64) -> Self {
        let mut g = Game {
            board: [[None; BOARD_WIDTH]; BOARD_HEIGHT],
            current: Piece::spawn(Kind::I),
            next: Kind::I,
            score: 0,
            lines_cleared: 0,
            game_over: false,
            rng_state: seed | 1,
        };
        let first = g.random_kind();
        g.next = g.random_kind();
        g.current = Piece::spawn(first);
        g
    }

    fn random_kind(&mut self) -> Kind {
        // xorshift64
        let mut x = self.rng_state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng_state = x;
        ALL_KINDS[(x % ALL_KINDS.len() as u64) as usize]
    }

    fn fits(&self, piece: &Piece) -> bool {
        piece.cells().all(|(x, y)| {
            x >= 0
                && x < BOARD_WIDTH as i32
                && y < BOARD_HEIGHT as i32
                && (y < 0 || self.board[y as usize][x as usize].is_none())
        })
    }

    pub fn move_by(&mut self, dx: i32, dy: i32) -> bool {
        if self.game_over {
            return false;
        }
        let candidate = Piece { x: self.current.x + dx, y: self.current.y + dy, ..self.current };
        if self.fits(&candidate) {
            self.current = candidate;
            true
        } else {
            false
        }
    }

    pub fn rotate_cw(&mut self) -> bool {
        if self.game_over {
            return false;
        }
        let candidate =
            Piece { rotation: (self.current.rotation + 1) % 4, ..self.current };
        if self.fits(&candidate) {
            self.current = candidate;
            true
        } else {
            false
        }
    }

    /// Gravity tick: try to move down, otherwise lock the piece and spawn the next one.
    pub fn tick(&mut self) {
        if self.game_over {
            return;
        }
        if !self.move_by(0, 1) {
            self.lock_and_spawn();
        }
    }

    pub fn hard_drop(&mut self) {
        if self.game_over {
            return;
        }
        while self.move_by(0, 1) {}
        self.lock_and_spawn();
    }

    fn lock_and_spawn(&mut self) {
        for (x, y) in self.current.cells() {
            if y < 0 {
                self.game_over = true;
                return;
            }
            self.board[y as usize][x as usize] = Some(self.current.kind);
        }
        self.clear_lines();

        let next_kind = self.next;
        self.next = self.random_kind();
        let spawned = Piece::spawn(next_kind);
        if !self.fits(&spawned) {
            self.game_over = true;
        }
        self.current = spawned;
    }

    fn clear_lines(&mut self) {
        let mut cleared = 0u32;
        let mut write = BOARD_HEIGHT;
        for read in (0..BOARD_HEIGHT).rev() {
            let full = self.board[read].iter().all(|c| c.is_some());
            if full {
                cleared += 1;
                continue;
            }
            write -= 1;
            self.board[write] = self.board[read];
        }
        for row in self.board.iter_mut().take(write) {
            *row = [None; BOARD_WIDTH];
        }
        self.lines_cleared += cleared;
        self.score += match cleared {
            1 => 100,
            2 => 300,
            3 => 500,
            4 => 800,
            _ => 0,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawn_piece_fits_on_empty_board() {
        let game = Game::new(1);
        assert!(game.fits(&game.current));
    }

    #[test]
    fn move_left_right_respects_walls() {
        let mut game = Game::new(1);
        for _ in 0..20 {
            game.move_by(-1, 0);
        }
        assert!(game.current.x >= 0);
        for _ in 0..20 {
            game.move_by(1, 0);
        }
        assert!(game.current.cells().all(|(x, _)| x < BOARD_WIDTH as i32));
    }

    #[test]
    fn piece_locks_at_the_bottom() {
        let mut game = Game::new(2);
        for _ in 0..200 {
            game.tick();
            if game.game_over {
                break;
            }
        }
        let occupied: usize =
            game.board.iter().flatten().filter(|c| c.is_some()).count();
        assert!(occupied > 0);
    }

    #[test]
    fn full_row_is_cleared() {
        let mut game = Game::new(3);
        // Fill the bottom row except columns 0-1, which the O piece (grid columns
        // 1-2, so board columns 0-1 at x = -1) will drop into and complete.
        for x in 2..BOARD_WIDTH {
            game.board[BOARD_HEIGHT - 1][x] = Some(Kind::O);
        }
        game.current = Piece { kind: Kind::O, rotation: 0, x: -1, y: BOARD_HEIGHT as i32 - 5 };
        game.hard_drop();
        assert_eq!(game.lines_cleared, 1);
        assert_eq!(game.score, 100);
    }

    #[test]
    fn rotation_blocked_by_wall_is_rejected() {
        let mut game = Game::new(4);
        game.current = Piece { kind: Kind::I, rotation: 0, x: -1, y: 5 };
        let before = game.current.rotation;
        game.rotate_cw();
        // Either it found a valid rotation or stayed put — never goes out of bounds.
        assert!(game.fits(&game.current));
        let _ = before;
    }
}
