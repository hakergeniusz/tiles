//! Pure 2048 game logic, independent of the UI.

use rand::Rng;

/// Board size (4x4 grid).
pub const SIZE: usize = 4;

/// The four possible moves.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

/// What changed on the board during a move, used to animate it.
#[derive(Debug, Clone)]
pub struct MoveEffect {
    /// Tiles that changed cell: ((from_row, from_col), (to_row, to_col)).
    /// A merging tile that already sits on the destination cell is left out —
    /// the old board draws it at the destination anyway.
    pub moves: Vec<((usize, usize), (usize, usize))>,
    /// Cells where two tiles merged (the destination cell).
    pub merged: Vec<(usize, usize)>,
    /// Score gained by the merges.
    pub gained: u32,
}

/// State of a single 2048 round.
pub struct Game {
    /// Tile values, `0` for empty cells, indexed `[row][col]`.
    pub board: [[u32; SIZE]; SIZE],
    /// Current score.
    pub score: u32,
    /// Whether a 2048 tile has been created.
    pub won: bool,
    /// Whether the player chose to keep playing after winning.
    pub keep_playing: bool,
    /// Whether no further move is possible.
    pub over: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

impl Game {
    /// Start a new round with two random tiles.
    pub fn new() -> Self {
        let mut game = Self {
            board: [[0; SIZE]; SIZE],
            score: 0,
            won: false,
            keep_playing: false,
            over: false,
        };
        game.spawn_tile();
        game.spawn_tile();
        game
    }

    /// Place a 2 (90 %) or 4 (10 %) tile on a random empty cell and return
    /// the cell it was placed on.
    pub fn spawn_tile(&mut self) -> Option<(usize, usize)> {
        let mut empty = Vec::new();
        for row in 0..SIZE {
            for col in 0..SIZE {
                if self.board[row][col] == 0 {
                    empty.push((row, col));
                }
            }
        }
        let (row, col) = *empty.get(rand::rng().random_range(0..empty.len()))?;

        let value = if rand::rng().random_bool(0.9) { 2 } else { 4 };
        self.board[row][col] = value;
        Some((row, col))
    }

    /// Slide and merge every line towards `dir`, then add the merge score.
    /// Reports what moved where so the UI can animate it. Returns `None`
    /// when the move would change nothing.
    pub fn move_dir_with_effects(&mut self, dir: Direction) -> Option<MoveEffect> {
        let mut moves = Vec::new();
        let mut merged = Vec::new();
        let mut gained = 0;
        let mut changed = false;

        for i in 0..SIZE {
            // Collect the non-zero tiles of the line in merge order,
            // together with their board coordinates.
            let mut tiles = Vec::new();
            for j in 0..SIZE {
                let (row, col) = coord_of(dir, i, j);
                if self.board[row][col] != 0 {
                    tiles.push((self.board[row][col], (row, col)));
                }
            }

            // Compact the line and merge equal neighbours once.
            let mut out = [0; SIZE];
            let mut read = 0;
            let mut write = 0;
            while read < tiles.len() {
                let dest = coord_of(dir, i, write);
                if read + 1 < tiles.len() && tiles[read].0 == tiles[read + 1].0 {
                    let value = tiles[read].0 * 2;
                    out[write] = value;
                    gained += value;
                    moves.push((tiles[read].1, dest));
                    moves.push((tiles[read + 1].1, dest));
                    merged.push(dest);
                    read += 2;
                } else {
                    out[write] = tiles[read].0;
                    moves.push((tiles[read].1, dest));
                    read += 1;
                }
                write += 1;
            }

            // Write the line back, remembering whether anything changed.
            for (j, &value) in out.iter().enumerate() {
                let (row, col) = coord_of(dir, i, j);
                if self.board[row][col] != value {
                    changed = true;
                }
                self.board[row][col] = value;
            }
        }

        if !changed {
            return None;
        }
        moves.retain(|&(from, to)| from != to);

        self.score += gained;
        if !self.won && self.board.iter().any(|row| row.contains(&2048)) {
            self.won = true;
        }

        Some(MoveEffect {
            moves,
            merged,
            gained,
        })
    }

    /// Whether any move can still change the board.
    pub fn can_move(&self) -> bool {
        for row in 0..SIZE {
            for col in 0..SIZE {
                let value = self.board[row][col];
                if value == 0 {
                    return true;
                }
                if row + 1 < SIZE && self.board[row + 1][col] == value {
                    return true;
                }
                if col + 1 < SIZE && self.board[row][col + 1] == value {
                    return true;
                }
            }
        }
        false
    }

    /// Whether the round is lost (no move possible).
    pub fn is_game_over(&self) -> bool {
        !self.can_move()
    }
}

/// Coordinates of slot `j` of line `i` for a move towards `dir`, such that
/// merging towards `dir` is merging towards the front of the line.
fn coord_of(dir: Direction, i: usize, j: usize) -> (usize, usize) {
    match dir {
        Direction::Left => (i, j),
        Direction::Right => (i, SIZE - 1 - j),
        Direction::Up => (j, i),
        Direction::Down => (SIZE - 1 - j, i),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_with(board: [[u32; SIZE]; SIZE]) -> Game {
        Game {
            board,
            score: 0,
            won: false,
            keep_playing: false,
            over: false,
        }
    }

    #[test]
    fn merge_slides_and_merges_one_pair() {
        let mut game = game_with([[2, 2, 2, 0], [0; 4], [0; 4], [0; 4]]);
        game.move_dir_with_effects(Direction::Left)
            .expect("valid move");
        assert_eq!(game.board[0], [4, 2, 0, 0]);
        assert_eq!(game.score, 4);
    }

    #[test]
    fn merge_two_pairs() {
        let mut game = game_with([[2, 2, 2, 2], [0; 4], [0; 4], [0; 4]]);
        game.move_dir_with_effects(Direction::Left)
            .expect("valid move");
        assert_eq!(game.board[0], [4, 4, 0, 0]);
        assert_eq!(game.score, 8);
    }

    #[test]
    fn merge_skips_gaps() {
        let mut game = game_with([[0, 4, 0, 4], [0; 4], [0; 4], [0; 4]]);
        game.move_dir_with_effects(Direction::Left)
            .expect("valid move");
        assert_eq!(game.board[0], [8, 0, 0, 0]);
        assert_eq!(game.score, 8);
    }

    #[test]
    fn merge_without_pair_just_slides() {
        let mut game = game_with([[0, 2, 0, 4], [0; 4], [0; 4], [0; 4]]);
        game.move_dir_with_effects(Direction::Left)
            .expect("valid move");
        assert_eq!(game.board[0], [2, 4, 0, 0]);
        assert_eq!(game.score, 0);
    }

    #[test]
    fn move_left_merges_towards_the_left() {
        let mut game = game_with([[2, 2, 0, 0], [0; 4], [0; 4], [0; 4]]);
        assert!(game.move_dir_with_effects(Direction::Left).is_some());
        assert_eq!(game.board[0], [4, 0, 0, 0]);
        assert_eq!(game.score, 4);
    }

    #[test]
    fn move_right_merges_towards_the_right() {
        let mut game = game_with([[2, 2, 4, 0], [0; 4], [0; 4], [0; 4]]);
        assert!(game.move_dir_with_effects(Direction::Right).is_some());
        assert_eq!(game.board[0], [0, 0, 4, 4]);
        assert_eq!(game.score, 4);
    }

    #[test]
    fn move_up_and_down_work_on_columns() {
        let rows = [[2, 0, 0, 0], [2, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]];

        let mut game = game_with(rows);
        assert!(game.move_dir_with_effects(Direction::Up).is_some());
        assert_eq!(game.board[0][0], 4);
        assert_eq!(game.score, 4);

        let mut game = game_with(rows);
        assert!(game.move_dir_with_effects(Direction::Down).is_some());
        assert_eq!(game.board[3][0], 4);
        assert_eq!(game.score, 4);
    }

    #[test]
    fn a_move_without_effect_is_rejected() {
        let mut game = game_with([[2, 0, 0, 0], [0; 4], [0; 4], [0; 4]]);
        assert!(game.move_dir_with_effects(Direction::Left).is_none());
        assert_eq!(game.score, 0);
    }

    #[test]
    fn effects_are_absent_for_a_move_without_effect() {
        let mut game = game_with([[2, 0, 0, 0], [0; 4], [0; 4], [0; 4]]);
        assert!(game.move_dir_with_effects(Direction::Left).is_none());
    }

    #[test]
    fn move_effects_describe_a_simple_merge() {
        let mut game = game_with([[2, 0, 0, 2], [0; 4], [0; 4], [0; 4]]);
        let effect = game
            .move_dir_with_effects(Direction::Left)
            .expect("valid move");
        // The moving tile slides onto the stationary one at column 0.
        assert_eq!(effect.moves, [((0, 3), (0, 0))]);
        assert_eq!(effect.merged, [(0, 0)]);
        assert_eq!(effect.gained, 4);
        assert_eq!(game.board[0], [4, 0, 0, 0]);
    }

    #[test]
    fn merge_with_a_stationary_partner_reports_only_the_mover() {
        let mut game = game_with([[2, 2, 0, 0], [0; 4], [0; 4], [0; 4]]);
        let effect = game
            .move_dir_with_effects(Direction::Left)
            .expect("valid move");
        assert_eq!(effect.moves, [((0, 1), (0, 0))]);
        assert_eq!(effect.merged, [(0, 0)]);
    }

    #[test]
    fn reaching_2048_wins() {
        let mut game = game_with([[1024, 1024, 0, 0], [0; 4], [0; 4], [0; 4]]);
        assert!(game.move_dir_with_effects(Direction::Left).is_some());
        assert!(game.won);
    }

    #[test]
    fn a_full_board_with_no_pairs_is_game_over() {
        let game = game_with([[2, 4, 2, 4], [4, 2, 4, 2], [2, 4, 2, 4], [4, 2, 4, 2]]);
        assert!(game.is_game_over());
    }

    #[test]
    fn a_full_board_with_pairs_can_still_move() {
        let game = game_with([[2, 2, 4, 8], [4, 8, 16, 32], [2, 4, 2, 4], [8, 4, 8, 4]]);
        assert!(!game.is_game_over());
    }
}
