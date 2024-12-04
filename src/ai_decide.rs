use super::proto::{Color, Move};
use crate::depth_first_search;
use crate::monte;
use crate::use_book;
use std::collections::HashMap;
use crate::bit_othello::Board;

pub fn decide(
    board: &Board,
    player_color: Color,
    kihu: &Vec<Move>,
    book_dict: &HashMap<String, String>,
) -> Move {
    let time_to_decide = 1400;
    let moves = board.valid_moves(player_color);
    if moves.is_empty() {
        Move::Pass
    } else if moves.len() == 1 {
        Move::Mv {
            x_ah: moves[0].0 as u32,
            y_18: moves[0].1 as u32,
        }
    } else {
        match use_book::decide(&kihu, &book_dict) {
            Some(mv) if board.check_valid_move(mv, player_color) => {
                println!("I use book");
                mv
            }
            _ => {
                if board.sum_stones() <= 42 {
                    monte::decide(board, player_color, time_to_decide)
                } else {
                    let max_eval;
                    let mv;
                    (mv, max_eval) =
                        depth_first_search::decide(board, player_color, time_to_decide);
                    if max_eval == None {
                        // 読みきれなかった場合
                        println!("I failed search all moves");
                        monte::decide(board, player_color, time_to_decide)
                    } else if max_eval == Some(-1) {
                        // 負け確定の場合
                        println!("I will lose");
                        let (mv_second, max_eval_second) = if board.sum_stones() >= 46 {
                            depth_first_search::perfect_read(board, player_color, time_to_decide)
                        } else {
                            (Move::GiveUp, None)
                        };
                        if max_eval_second == None {
                            monte::decide(board, player_color, time_to_decide)
                        } else {
                            println!("predicted diff stones: {}", max_eval_second.unwrap());
                            mv_second
                        }
                    } else {
                        // 引き分けか勝ち確定の場合
                        println!("You will lose");
                        // let (mv_second, max_eval_second) =
                        //     depth_first_search::perfect_read(board, player_color, time_to_decide);
                        let (mv_second, max_eval_second) = if board.sum_stones() >= 46 {
                            depth_first_search::perfect_read(board, player_color, time_to_decide)
                        } else {
                            (Move::GiveUp, None)
                        };
                        if max_eval_second == None {
                            mv
                        } else {
                            println!("predicted diff stones: {}", max_eval_second.unwrap());
                            mv_second
                        }
                    }
                }
            }
        }
    }
}
