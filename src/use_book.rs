use super::book::DATA;
// use super::large_book::DATA;
use super::proto::Move;
use std::collections::HashMap;

use crate::bit_othello::Pos;

fn str2pos(s: &str) -> Pos {
    let x = s.chars().nth(0).unwrap() as u32 - 'A' as u32 + 1;
    let y = s.chars().nth(1).unwrap() as u32 - '1' as u32 + 1;
    (x as usize, y as usize)
}

fn move2str(m: Move) -> String {
    if let Move::Mv { x_ah, y_18 } = m {
        format!(
            "{}{}",
            ('A' as u8 + x_ah as u8 - 1) as char,
            ('1' as u8 + y_18 as u8 - 1) as char
        )
    } else {
        "".to_string()
    }
}

fn rotate_pos(pos: Pos, rot: u32) -> Pos {
    let (x, y) = pos;
    match rot {
        0 => (x, y),
        1 => (9 - y, 9 - x),
        2 => (9 - x, 9 - y),
        3 => (y, x),
        _ => (x, y),
    }
}

pub fn initialize_book_dict() -> HashMap<String, String> {
    let lines: Vec<&str> = DATA.split('\n').collect();
    let mut book_dict: HashMap<String, String> = HashMap::new();

    for line in lines {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() == 2 {
            book_dict.insert(parts[0].to_string(), parts[1].to_string());
        }
    }

    book_dict
}

pub fn decide(kihu: &Vec<Move>, book_dict: &HashMap<String, String>) -> Option<Move> {
    let kihu_str: String = kihu.iter().map(|m| move2str(*m)).collect();
    let mut pos: Option<Move> = None;
    for rot in 0..4 {
        let kihu_rot = kihu_str.chars().collect::<Vec<char>>();
        let mut kihu_rot_str = String::new();
        for i in 0..kihu_rot.len() / 2 {
            let pos = str2pos(&kihu_str[i * 2..i * 2 + 2]);
            let pos_rot = rotate_pos(pos, rot);
            kihu_rot_str.push(('A' as u8 + pos_rot.0 as u8 - 1) as char);
            kihu_rot_str.push(('1' as u8 + pos_rot.1 as u8 - 1) as char);
        }
        if let Some(value) = book_dict.get(&kihu_rot_str) {
            let next_move = str2pos(value);
            let rotated_next_move = rotate_pos(next_move, rot);
            pos = Some(Move::Mv {
                x_ah: rotated_next_move.0 as u32,
                y_18: rotated_next_move.1 as u32,
            });
        }
    }
    return pos;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_initialize_book_dict() {
        let start = Instant::now();
        let book_dict = initialize_book_dict();
        let elapsed = start.elapsed();
        let len_dict = book_dict.len();
        let mut count = 0;
        for (key, value) in book_dict.iter() {
            if key.len() == 54 {
                println!("{}:{}", key, value);
                count += 1;
            }
        }
        println!("count: {}", count);
        println!("len: {}", len_dict);
        println!("elapsed: {:?}", elapsed);
        let move_list = vec![Move::Mv { x_ah: 4, y_18: 3 }];
        let next_move = decide(&move_list, &book_dict);
        println!("{:?}", next_move);
    }
}
