use super::proto::{Color, Move};
use rand::Rng;

use crate::bit_othello::{get_corner_list, Board};

fn minus_tuple(a: (i32, i32)) -> (i32, i32) {
    (-a.0, -a.1)
}

const USE_MOBILITY: u64 = 64;
const USE_MOBILITY_FROAT: f64 = USE_MOBILITY as f64;
const MOBILITY_SCALE: f64 = 0.3;
const EXPAND_THRESHOLD: i32 = 5;

#[derive(Clone)]
pub struct MonteNode {
    pub board: Board,
    pub color: Color,
    // pub parent: Option<Box<MonteNode>>,
    pub children: Vec<MonteNode>,
    pub is_expanded: bool,  // 展開済みかどうか
    pub wins: i32,          // 勝利数
    pub visits: i32,        // このノードを調べた回数
    pub is_game_end: bool,  // ゲーム終了ノードかどうか
    pub prev_is_skip: bool, // 1つ前の手がパスかどうか
    // pub is_root: bool,      // ルートノードかどうか
    pub prev_move: Option<Move>,
    pub canput_diff: i32,
    pub mobility: i32,
    pub stone_sum: u64,
}

impl MonteNode {
    pub fn new(board: Board, color: Color, prev_move: Option<Move>, stone_sum: u64) -> Self {
        let canput_diff = board.canput_diff(color);
        Self {
            board,
            color,
            children: vec![],
            is_expanded: false,
            wins: 0,
            visits: 0,
            is_game_end: false,
            prev_is_skip: false,
            prev_move: prev_move,
            canput_diff: canput_diff,
            mobility: 0,
            stone_sum: stone_sum,
        }
    }

    pub fn play_out(&mut self) -> (i32, i32) {
        let mut rng = rand::thread_rng();
        let mut board = self.board.clone();
        let mut color = self.color;
        let mut is_passed = false;
        // self.visits += 1;
        if !self.is_expanded {
            if self.visits > EXPAND_THRESHOLD {
                self.expand();
            }
        }
        if self.is_expanded {
            let mut max_ucb = 0.0;
            let mut max_ucb_index = 0;
            if self.is_game_end {
                let ret = (board.win_or_lose(self.color), 0); // このノードの勝敗 ゲームの決着がついているので手数差は0

                // self.wins += ret.0;
                self.add(ret.0, 0);
                return ret;
            }
            for (i, child) in self.children.iter().enumerate() {
                let ucb = if child.visits == 0 {
                    f64::INFINITY
                } else {
                    let ucb = calculate_ucb(
                        child.wins,
                        child.visits,
                        self.visits,
                        child.mobility,
                        child.stone_sum,
                    );
                    ucb
                };
                if ucb > max_ucb {
                    max_ucb = ucb;
                    max_ucb_index = i;
                }
            }
            let child = &mut self.children[max_ucb_index];
            let result = minus_tuple(child.play_out());
            // self.wins += result.0;
            // self.mobility += result.1;
            self.add(result.0, result.1);
            return result;
        }
        loop {
            let moves = board.valid_moves(color);
            if moves.is_empty() {
                if is_passed {
                    break;
                }
                is_passed = true;
                color = color.opposite();
                continue;
            }
            is_passed = false;
            // let m = moves[rng.gen_range(0..moves.len())];
            let moves_len = moves.len();
            let m = if moves_len == 1 {
                moves[0]
            } else {
                let corner_list = get_corner_list(&moves);
                if !corner_list.is_empty() && rng.gen_bool(0.6) {
                    corner_list[rng.gen_range(0..corner_list.len())]
                } else if rng.gen_bool(0.6) {
                    speedy_decide(&board, color)
                } else {
                    moves[rng.gen_range(0..moves_len)]
                }
            };
            board.do_move(
                Move::Mv {
                    x_ah: m.0 as u32,
                    y_18: m.1 as u32,
                },
                color,
            );
            color = color.opposite();
        }
        let ret = board.win_or_lose(self.color);
        // self.wins += ret;
        // self.mobility += self.canput_diff;
        self.add(ret, self.canput_diff);
        (ret, self.canput_diff)
    }

    pub fn add(&mut self, wins: i32, mobility: i32) {
        self.wins += wins;
        self.visits += 1;
        self.mobility += mobility;
    }

    pub fn expand(&mut self) {
        let moves = self.board.valid_moves(self.color);
        self.is_expanded = true;
        if moves.is_empty() {
            if self.prev_is_skip {
                self.is_game_end = true;
                return;
            }
            let mut child = MonteNode::new(
                self.board.clone(),
                self.color.opposite(),
                // Some(Box::new(self.clone())),
                Some(Move::Pass),
                self.stone_sum,
            );
            child.prev_is_skip = true;
            self.children.push(child);
            return;
        }
        for m in moves {
            let mut board = self.board.clone();
            let selected_move = Move::Mv {
                x_ah: m.0 as u32,
                y_18: m.1 as u32,
            };
            board.do_move(selected_move, self.color);
            let child = MonteNode::new(
                board,
                self.color.opposite(),
                // Some(Box::new(self.clone())),
                Some(selected_move),
                self.stone_sum + 1,
            );
            self.children.push(child);
        }
    }
}

fn calculate_ucb(wins: i32, visits: i32, parent_visits: i32, mobility: i32, stone_sum: u64) -> f64 {
    if visits == 0 {
        f64::INFINITY
    } else {
        let win_rate = (visits - wins) as f64 / visits as f64;
        let mobility_point = if stone_sum < USE_MOBILITY {
            mobility as f64 * (1.0 - (stone_sum as f64 / USE_MOBILITY_FROAT)) / visits as f64
        } else {
            0.0
        };
        let ucb = win_rate - mobility_point * MOBILITY_SCALE
            + 2.0 * (2.0 * (parent_visits as f64).ln() / visits as f64).sqrt();
        ucb
    }
}

fn speedy_decide(board: &Board, color: Color) -> (usize, usize) {
    let canput = board.valid_moves(color);
    let mut max_score = -1000;
    let mut max_score_index = 0;
    for (i, pos) in canput.iter().enumerate() {
        let mut board = board.clone();
        let m = Move::Mv {
            x_ah: pos.0 as u32,
            y_18: pos.1 as u32,
        };
        board.do_move(m, color);
        let score = board.canput_diff(color);
        if score > max_score {
            max_score = score;
            max_score_index = i;
        }
    }
    canput[max_score_index]
}

fn calc_max_depth(root: &MonteNode) -> i32 {
    let mut max_depth = 0;
    for child in root.children.iter() {
        let depth = calc_max_depth(child);
        if depth > max_depth {
            max_depth = depth;
        }
    }
    max_depth + 1
}

pub fn decide(board: &Board, color: Color, assigned_time_ms: i32) -> Move {
    let mut root = MonteNode::new(board.clone(), color, None, board.sum_stones());
    root.expand();
    // let mut root = root;
    let start = std::time::Instant::now();
    while start.elapsed().as_millis() < assigned_time_ms as u128 {
        root.play_out();
    }
    let mut max_visits = -1;
    let mut max_visits_index = 0;
    let mut sum_visits = 0;
    for (i, child) in root.children.iter().enumerate() {
        let winrate = child.wins as f64 / child.visits as f64;
        print!(
            "Move: {}, Max Depth:{}, n:{}, Winrate: {} tekazu: {}\n",
            child.prev_move.unwrap(),
            calc_max_depth(child),
            child.visits,
            winrate * (-1.0 / 2.0) + 1.0 / 2.0,
            if child.visits == 0 {
                0.0
            } else {
                -child.mobility as f64 / child.visits as f64
            }
        );
        sum_visits += child.visits;
        if child.visits > max_visits {
            max_visits = child.visits;
            max_visits_index = i;
        }
    }
    print!("Sum Visits: {}\n", sum_visits);
    let child = &root.children[max_visits_index];
    let ret = child.prev_move.unwrap();
    ret
}
