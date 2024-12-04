use rayon::iter::{IntoParallelIterator, ParallelIterator};

use super::proto::{Color, Move};

// use crate::play::Board;
use crate::bit_othello::Board;

fn reverse_value(value: Option<i32>) -> Option<i32> {
    match value {
        Some(x) => Some(-x),
        None => None,
    }
}

pub fn calc_by_dfs(
    board: &Board,
    color: Color,
    prev_move: Option<Move>,
    start_time: &std::time::Instant,
    assigned_time_ms: i32,
    stone_num: u64,
) -> Option<i32> {
    if start_time.elapsed().as_millis() > assigned_time_ms as u128 {
        return None; // Timeout
    }
    let moves = board.valid_moves(color);
    if moves.is_empty() {
        if prev_move == Some(Move::Pass) {
            Some(board.win_or_lose(color))
        } else {
            let next_color = color.opposite();
            reverse_value(calc_by_dfs(
                board,
                next_color,
                Some(Move::Pass),
                start_time,
                assigned_time_ms,
                stone_num,
            ))
        }
    } else {
        let mut max_eval = -64;
        if stone_num < 60 {
            let mut board_list: Vec<(Board, i32, Move)> = vec![];
            for m in moves {
                let mut board = board.clone();
                let selected_move = Move::Mv {
                    x_ah: m.0 as u32,
                    y_18: m.1 as u32,
                };
                board.do_move(selected_move, color);
                let canput_diff = board.canput_diff(color);
                board_list.push((board, canput_diff, selected_move));
                // let ret = reverse_value(calc_by_dfs(
                //     &board,
                //     color.opposite(),
                //     Some(selected_move),
                //     start_time,
                //     assigned_time_ms,
                // ));
                // if ret == None {
                //     return None;
                // }
                // let ret = ret.unwrap();
                // if ret > max_eval {
                //     max_eval = ret;
                // }
                // if ret == 1 {
                //     return Some(1);
                // }
            }
            board_list.sort_by(|a, b| b.1.cmp(&a.1));
            for (board, _, selected_move) in board_list {
                let ret = reverse_value(calc_by_dfs(
                    &board,
                    color.opposite(),
                    Some(selected_move),
                    start_time,
                    assigned_time_ms,
                    stone_num + 1,
                ));
                if let Some(ret_raw) = ret {
                    if ret_raw > max_eval {
                        max_eval = ret_raw;
                    }
                    if ret_raw == 1 {
                        return Some(1);
                    }
                } else {
                    return None;
                }
            }
        } else {
            for m in moves {
                let mut board = board.clone();
                let selected_move = Move::Mv {
                    x_ah: m.0 as u32,
                    y_18: m.1 as u32,
                };
                board.do_move(selected_move, color);
                let ret = reverse_value(calc_by_dfs(
                    &board,
                    color.opposite(),
                    Some(selected_move),
                    start_time,
                    assigned_time_ms,
                    stone_num + 1,
                ));
                if let Some(ret_raw) = ret {
                    if ret_raw > max_eval {
                        max_eval = ret_raw;
                    }
                    if ret_raw == 1 {
                        return Some(1);
                    }
                } else {
                    return None;
                }
            }
        }

        Some(max_eval)
    }
}

pub fn perfect_read_dfs(
    board: &Board,
    color: Color,
    prev_move: Option<Move>,
    start_time: &std::time::Instant,
    assigned_time_ms: i32,
) -> Option<i32> {
    if start_time.elapsed().as_millis() > assigned_time_ms as u128 {
        return None; // Timeout
    }
    let moves = board.valid_moves(color);
    if moves.is_empty() {
        if prev_move == Some(Move::Pass) {
            Some(board.diff_stones(color))
        } else {
            let next_color = color.opposite();
            reverse_value(perfect_read_dfs(
                board,
                next_color,
                Some(Move::Pass),
                start_time,
                assigned_time_ms,
            ))
        }
    } else {
        let mut max_eval = -64;
        for m in moves {
            let mut board = board.clone();
            let selected_move = Move::Mv {
                x_ah: m.0 as u32,
                y_18: m.1 as u32,
            };
            board.do_move(selected_move, color);
            let ret = reverse_value(perfect_read_dfs(
                &board,
                color.opposite(),
                Some(selected_move),
                start_time,
                assigned_time_ms,
            ));
            if let Some(ret_raw) = ret {
                if ret_raw > max_eval {
                    max_eval = ret_raw;
                }
            } else {
                return None;
            }
        }
        Some(max_eval)
    }
}

pub fn decide(board: &Board, color: Color, assigned_time_ms: i32) -> (Move, Option<i32>) {
    let moves = board.valid_moves(color);
    let moves_len = moves.len();
    let mut max_eval = -64;
    let mut best_move = Move::Pass;
    let start_time = std::time::Instant::now();
    let stone_num = board.sum_stones();

    let results: Vec<_> = moves
        .into_par_iter()
        .filter_map(|m| {
            let mut board = board.clone();
            let selected_move = Move::Mv {
                x_ah: m.0 as u32,
                y_18: m.1 as u32,
            };
            board.do_move(selected_move, color);
            let ret = reverse_value(calc_by_dfs(
                &board,
                color.opposite(),
                Some(selected_move),
                &start_time,
                assigned_time_ms,
                stone_num + 1,
            ));
            ret.map(|ret| (selected_move, ret))
        })
        .collect();

    let results_len = results.len();

    for (selected_move, ret) in results {
        if ret > max_eval {
            max_eval = ret;
            best_move = selected_move;
        }
        if ret == 1 {
            return (best_move, Some(max_eval));
        }
    }

    if max_eval == 0 {
        return (best_move, Some(max_eval));
    }
    if results_len < moves_len {
        return (Move::GiveUp, None);
    }
    (best_move, Some(max_eval))

    // let result = moves.into_par_iter().filter_map(|m| {
    //     let mut board = board.clone();
    //     let selected_move = Move::Mv {
    //         x_ah: m.0 as u32,
    //         y_18: m.1 as u32,
    //     };
    //     board.do_move(selected_move, color);
    //     let ret = reverse_value(calc_by_dfs(
    //         &board,
    //         color.opposite(),
    //         Some(selected_move),
    //         &start_time,
    //         assigned_time_ms,
    //     ));
    //     if ret == None {
    //         return None;
    //     }
    //     let ret = ret.unwrap();
    //     if ret > max_eval {
    //         max_eval = ret;
    //         best_move = selected_move;
    //     }
    //     if ret == 1 {
    //         return Some((best_move, Some(max_eval)));
    //     }
    //     None
    // });

    // for m in moves {
    //     let mut board = board.clone();
    //     let selected_move = Move::Mv {
    //         x_ah: m.0 as u32,
    //         y_18: m.1 as u32,
    //     };
    //     board.do_move(selected_move, color);
    //     let ret = reverse_value(calc_by_dfs(
    //         &board,
    //         color.opposite(),
    //         Some(selected_move),
    //         &start_time,
    //         assigned_time_ms,
    //     ));
    //     if ret == None {
    //         return (Move::GiveUp, None);
    //     }
    //     let ret = ret.unwrap();
    //     if ret > max_eval {
    //         max_eval = ret;
    //         best_move = selected_move;
    //     }
    //     if ret == 1 {
    //         break;
    //     }
    // }
}

pub fn perfect_read(board: &Board, color: Color, assigned_time_ms: i32) -> (Move, Option<i32>) {
    let moves = board.valid_moves(color);
    let mut max_eval = -65;
    let mut best_move = Move::Pass;
    let start_time = std::time::Instant::now();

    let moves_len = moves.len();

    let results: Vec<(Move, i32)> = moves
        .into_par_iter()
        .filter_map(|m| {
            let mut board = board.clone();
            let selected_move = Move::Mv {
                x_ah: m.0 as u32,
                y_18: m.1 as u32,
            };
            board.do_move(selected_move, color);
            let ret = reverse_value(perfect_read_dfs(
                &board,
                color.opposite(),
                Some(selected_move),
                &start_time,
                assigned_time_ms,
            ));
            ret.map(|ret| (selected_move, ret))
        })
        .collect();

    let results_len = results.len();
    if results_len < moves_len {
        return (Move::GiveUp, None);
    }
    for (selected_move, ret) in results {
        if ret > max_eval {
            max_eval = ret;
            best_move = selected_move;
        }
    }

    // for m in moves {
    //     let mut board = board.clone();
    //     let selected_move = Move::Mv {
    //         x_ah: m.0 as u32,
    //         y_18: m.1 as u32,
    //     };
    //     board.do_move(selected_move, color);
    //     let ret = reverse_value(perfect_read_dfs(
    //         &board,
    //         color.opposite(),
    //         Some(selected_move),
    //         &start_time,
    //         assigned_time_ms,
    //     ));
    //     if let Some(ret_raw) = ret {
    //         if ret_raw > max_eval {
    //             max_eval = ret_raw;
    //             best_move = selected_move;
    //         }
    //     } else {
    //         return (Move::GiveUp, None);
    //     }
    // }
    (best_move, Some(max_eval))
}
