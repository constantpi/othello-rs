use super::proto::{Color, Move};
#[cfg(test)]
use rand::Rng;
use std::fmt::{self, Display, Formatter};

pub struct InitGame {
    pub opponent_name: String,
    pub assigned_time_ms: i32,
}

#[derive(Clone)]
pub struct Board {
    pub black: u64,
    pub white: u64,
}

pub type Pos = (usize, usize);

impl Board {
    pub fn new() -> Self {
        Self {
            black: 0x0000000810000000,
            white: 0x0000001008000000,
        }
    }

    fn can_put(&self, color: Color) -> u64 {
        let p;
        let o;
        if color == Color::Black {
            p = self.black;
            o = self.white;
        } else {
            p = self.white;
            o = self.black;
        };
        let horizontal = o & 0x7e7e7e7e7e7e7e7e;
        let vertical = o & 0x00ffffffffffff00;
        let allside = o & 0x007e7e7e7e7e7e00;
        let blank = !(p | o);

        // left
        let mut temp = horizontal & (p << 1);
        temp |= horizontal & (temp << 1);
        temp |= horizontal & (temp << 1);
        temp |= horizontal & (temp << 1);
        temp |= horizontal & (temp << 1);
        temp |= horizontal & (temp << 1);
        let mut ans = (temp << 1) & blank;

        // right
        temp = horizontal & (p >> 1);
        temp |= horizontal & (temp >> 1);
        temp |= horizontal & (temp >> 1);
        temp |= horizontal & (temp >> 1);
        temp |= horizontal & (temp >> 1);
        temp |= horizontal & (temp >> 1);
        ans |= (temp >> 1) & blank;

        // up
        temp = vertical & (p << 8);
        temp |= vertical & (temp << 8);
        temp |= vertical & (temp << 8);
        temp |= vertical & (temp << 8);
        temp |= vertical & (temp << 8);
        temp |= vertical & (temp << 8);
        ans |= (temp << 8) & blank;

        // down
        temp = vertical & (p >> 8);
        temp |= vertical & (temp >> 8);
        temp |= vertical & (temp >> 8);
        temp |= vertical & (temp >> 8);
        temp |= vertical & (temp >> 8);
        temp |= vertical & (temp >> 8);
        ans |= (temp >> 8) & blank;

        // up-left
        temp = allside & (p << 9);
        temp |= allside & (temp << 9);
        temp |= allside & (temp << 9);
        temp |= allside & (temp << 9);
        temp |= allside & (temp << 9);
        temp |= allside & (temp << 9);
        ans |= (temp << 9) & blank;

        // down-right
        temp = allside & (p >> 9);
        temp |= allside & (temp >> 9);
        temp |= allside & (temp >> 9);
        temp |= allside & (temp >> 9);
        temp |= allside & (temp >> 9);
        temp |= allside & (temp >> 9);
        ans |= (temp >> 9) & blank;

        // up-right
        temp = allside & (p << 7);
        temp |= allside & (temp << 7);
        temp |= allside & (temp << 7);
        temp |= allside & (temp << 7);
        temp |= allside & (temp << 7);
        temp |= allside & (temp << 7);
        ans |= (temp << 7) & blank;

        // down-left
        temp = allside & (p >> 7);
        temp |= allside & (temp >> 7);
        temp |= allside & (temp >> 7);
        temp |= allside & (temp >> 7);
        temp |= allside & (temp >> 7);
        temp |= allside & (temp >> 7);
        ans |= (temp >> 7) & blank;

        ans
    }

    pub fn canput_diff(&self, color: Color) -> i32 {
        let black = self.can_put(Color::Black);
        let white = self.can_put(Color::White);
        let black_count = bit_count(black);
        let white_count = bit_count(white);
        if black_count == 0 && white_count == 0 {
            64 * self.win_or_lose(color)
        } else if color == Color::Black {
            black_count as i32 - white_count as i32
        } else {
            white_count as i32 - black_count as i32
        }
    }

    pub fn do_move(&mut self, m: Move, color: Color) {
        let p;
        let o;
        if color == Color::Black {
            p = &mut self.black;
            o = &mut self.white;
        } else {
            p = &mut self.white;
            o = &mut self.black;
        };
        if let Move::Mv { x_ah, y_18 } = m {
            let x = x_ah as usize;
            let y = y_18 as usize;
            let pos = (x - 1) * 8 + y - 1;
            let pos_bit = 1 << pos;
            let mut rev_temp;
            let mut rev = 0;
            for i in 0..8 {
                rev_temp = 0;
                let mut mask = transfer(pos_bit, i);
                while mask != 0 && (mask & *o) != 0 {
                    rev_temp |= mask;
                    mask = transfer(mask, i);
                }
                if (mask & *p) != 0 {
                    rev |= rev_temp;
                }
            }
            *p ^= pos_bit | rev;
            *o ^= rev;
        }
    }

    pub fn valid_moves(&self, color: Color) -> Vec<Pos> {
        let mut ret = vec![];
        let can_put = self.can_put(color);
        for i in 0..64 {
            if can_put & (1 << i) != 0 {
                ret.push((i / 8 + 1, i % 8 + 1));
            }
        }
        ret
    }

    pub fn count_stones(&self) -> (u64, u64) {
        (bit_count(self.black), bit_count(self.white))
    }

    pub fn win_or_lose(&self, color: Color) -> i32 {
        let (black, white) = self.count_stones();
        if color == Color::Black {
            if black > white {
                1
            } else if black < white {
                -1
            } else {
                0
            }
        } else {
            if black > white {
                -1
            } else if black < white {
                1
            } else {
                0
            }
        }
    }

    pub fn sum_stones(&self) -> u64 {
        let (black, white) = self.count_stones();
        black + white
    }

    pub fn diff_stones(&self, color: Color) -> i32 {
        let (black, white) = self.count_stones();
        if color == Color::Black {
            black as i32 - white as i32
        } else {
            white as i32 - black as i32
        }
    }

    pub fn check_valid_move(&self, m: Move, color: Color) -> bool {
        let valid_moves = self.valid_moves(color);
        if valid_moves.is_empty() && m == Move::Pass {
            true
        } else if let Move::Mv { x_ah, y_18 } = m {
            let x = x_ah as usize;
            let y = y_18 as usize;
            valid_moves.contains(&(x, y))
        } else {
            false
        }
    }
}

pub fn bit_count(x: u64) -> u64 {
    let mut x = x;
    x -= (x >> 1) & 0x5555555555555555;
    x = (x & 0x3333333333333333) + ((x >> 2) & 0x3333333333333333);
    x = (x + (x >> 4)) & 0x0f0f0f0f0f0f0f0f;
    x = x + (x >> 8);
    x = x + (x >> 16);
    x = x + (x >> 32);
    x & 0x7f
}

pub fn get_corner_list(valid_moves: &Vec<Pos>) -> Vec<Pos> {
    let mut ret = vec![];
    for &m in valid_moves {
        if (m.0 == 1 || m.0 == 8) && (m.1 == 1 || m.1 == 8) {
            ret.push(m);
        }
    }
    ret
}

fn transfer(put: u64, dir: u32) -> u64 {
    match dir {
        0 => (put << 8) & 0xffffffffffffff00, // up
        1 => (put << 7) & 0x7f7f7f7f7f7f7f00, // up-right
        2 => (put >> 1) & 0x7f7f7f7f7f7f7f7f, // right
        3 => (put >> 9) & 0x007f7f7f7f7f7f7f, // down-right
        4 => (put >> 8) & 0x00ffffffffffffff, // down
        5 => (put >> 7) & 0x00fefefefefefefe, // down-left
        6 => (put << 1) & 0xfefefefefefefefe, // left
        7 => (put << 9) & 0xfefefefefefefe00, // up-left
        _ => 0,
    }
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        const WHITE: char = 'O';
        const BLACK: char = 'X';
        writeln!(f, " |A B C D E F G H ")?;
        writeln!(f, "-+----------------")?;
        for j in 0..8 {
            write!(f, "{}|", j + 1)?;
            for i in 0..8 {
                let mark = if self.black & (1 << (i * 8 + j)) != 0 {
                    BLACK
                } else if self.white & (1 << (i * 8 + j)) != 0 {
                    WHITE
                } else {
                    ' '
                };
                write!(f, "{mark} ")?;
            }
            writeln!(f)?;
        }
        writeln!(f, "  ({BLACK}: Black, {WHITE}: White)")
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_board_fmt() {
//         let b = Board::new();
//         println!("{b}");
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_game() {
        let game = InitGame {
            opponent_name: String::from("Opponent"),
            assigned_time_ms: 60000,
        };
        assert_eq!(game.opponent_name, "Opponent");
        assert_eq!(game.assigned_time_ms, 60000);
    }

    #[test]
    fn test_board_initialization() {
        let board = Board {
            black: 0x0000000810000000,
            white: 0x0000001008000000,
        };
        assert_eq!(board.black, 0x0000000810000000);
        assert_eq!(board.white, 0x0000001008000000);
    }

    #[test]
    fn test_can_put() {
        let board = Board {
            black: 0x0000000810000000,
            white: 0x0000001008000000,
        };
        let can_put = board.can_put(Color::Black);
        println!("canput{:016x}", can_put);
    }

    // #[test]
    // fn test_playout() {
    //     let mut board = Board {
    //         black: 0x0000000810000000,
    //         white: 0x0000001008000000,
    //     };
    //     let mut color = Color::Black;
    //     let mut moves = vec![];
    //     for i in 0..60 {
    //         let valid_moves = board.valid_moves(color);
    //         println!("{},{:?}", i, valid_moves);

    //         if valid_moves.is_empty() {
    //             color = color.opposite();
    //             continue;
    //         }
    //         let m = valid_moves[rand::thread_rng().gen_range(0..valid_moves.len())];
    //         moves.push(m);
    //         board.do_move(
    //             Move::Mv {
    //                 x_ah: m.0 as u32,
    //                 y_18: m.1 as u32,
    //             },
    //             color,
    //         );
    //         color = color.opposite();
    //         println!("{:}", board);
    //     }
    //     println!("{:?}", moves);
    //     println!("{:?}", board.count_stones());
    //     println!("{:?}", board.win_or_lose(Color::Black));
    // }

    // 他のテスト関数をここに追加
}
