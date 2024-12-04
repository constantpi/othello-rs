use getopts::Options;
use std::collections::HashMap;
use std::env;
use std::fmt::Display;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::process;
use thiserror::Error;

mod ai_decide;
mod bit_othello;
mod book;
mod command_parser;
mod depth_first_search;
mod monte;
mod proto;
mod use_book;
use crate::bit_othello::{Board, InitGame};
use crate::proto::{Color, Move, PlayerStat, RecvCommand, SendCommand, Wl};

#[derive(Debug, Error)]
enum Error {
    #[error("couldn't read/write via IO stream `{0}`")]
    IO(#[from] std::io::Error),
    #[error("parse error: {0:?}")]
    Parse(String),
    #[error("received invalid command `{0:?}`")]
    Recv(RecvCommand),
}

type Result<T, E = Error> = std::result::Result<T, E>;

struct MyOptions {
    socket_addr: SocketAddr,
    player: String,
    verbose: bool,
}

struct Logger {
    enabled: bool,
}

impl Logger {
    fn new(opt: &MyOptions) -> Self {
        Self {
            enabled: opt.verbose,
        }
    }
    fn received(&self, s: &str) {
        if self.enabled {
            print!("Received: {s}");
        }
    }
    fn sent(&self, s: &str) {
        if self.enabled {
            print!("Sent: {s}");
        }
    }
    fn log(&self, s: impl Display) {
        if self.enabled {
            print!("{s}");
        }
    }
}

enum State {
    WaitStart,
    MyTurn(Option<InitGame>),
    OpTurn(Option<InitGame>),
    EndGame {
        result: Wl,
        your_stone_count: u32,
        opponent_stone_count: u32,
        reason: String,
    },
    Exit {
        stat: Vec<PlayerStat>,
    },
}

fn print_usage(program: &str, opts: &Options) -> ! {
    let brief = format!("Usage: {program} -H HOST -p PORT -n PLAYERNAME");
    print!("{}", opts.usage(&brief));
    process::exit(0);
}

fn parse_args() -> MyOptions {
    let args: Vec<String> = env::args().collect();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optopt("H", "host", "set server host", "HOST");
    opts.optopt("p", "port", "set server port", "PORT");
    opts.optopt("n", "name", "set player name", "PLAYERNAME");
    opts.optflag("v", "verbose", "verbose output");
    opts.optflag("h", "help", "print this help menu");

    let matches = opts.parse(&args[1..]).unwrap_or_else(|fail| {
        println!("{fail}");
        print_usage(program, &opts);
    });
    if matches.opt_present("h") {
        print_usage(program, &opts);
    }

    let host = matches
        .opt_str("H")
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let port = matches.opt_str("p").map_or(3000, |s| s.parse().unwrap());

    let addr = (host, port)
        .to_socket_addrs()
        .expect("hostname must be known")
        .next()
        .expect("hostname must be valid");

    MyOptions {
        socket_addr: addr,
        player: matches.opt_str("n").unwrap_or_else(|| "Anon.".to_string()),
        verbose: matches.opt_present("v"),
    }
}

fn receive_command(reader: &mut BufReader<&TcpStream>, logger: &mut Logger) -> Result<RecvCommand> {
    let mut buf = String::new();
    reader.read_line(&mut buf)?;
    logger.received(&buf);
    let rec = command_parser::parse(buf.as_str()).map_err(Error::Parse)?;
    Ok(rec)
}

fn send_command(
    writer: &mut BufWriter<&TcpStream>,
    logger: &mut Logger,
    command: &SendCommand,
) -> Result<()> {
    let s = command.to_string();
    writer.write_all(s.as_bytes())?;
    writer.flush()?;
    logger.sent(&s);
    Ok(())
}

fn wait_start(reader: &mut BufReader<&TcpStream>, logger: &mut Logger) -> Result<State> {
    match receive_command(reader, logger)? {
        RecvCommand::Bye { stat } => {
            println!("Finished connection.");
            Ok(State::Exit { stat })
        }
        RecvCommand::Start {
            color: Color::Black,
            opponent_name,
            assigned_time_ms,
        } => Ok(State::MyTurn(Some(InitGame {
            opponent_name,
            assigned_time_ms,
        }))),
        RecvCommand::Start {
            color: Color::White,
            opponent_name,
            assigned_time_ms,
        } => Ok(State::OpTurn(Some(InitGame {
            opponent_name,
            assigned_time_ms,
        }))),
        r => Err(Error::Recv(r)),
    }
}

fn print_scores(logger: &mut Logger, stat: impl Iterator<Item = PlayerStat>) {
    let v: Vec<_> = stat.map(|player_stat| format!("{player_stat}\n")).collect();
    logger.log(v.concat());
}

fn my_move(
    reader: &mut BufReader<&TcpStream>,
    writer: &mut BufWriter<&TcpStream>,
    logger: &mut Logger,
    board: &mut Board,
    player_color: Color,
    assigned_time_ms: &mut i32,
    book_dict: &HashMap<String, String>,
    kihu: &mut Vec<Move>,
) -> Result<State> {
    let mv = ai_decide::decide(board, player_color, &kihu, &book_dict);
    kihu.push(mv);
    // let mv = board.decide_move(player_color, *assigned_time_ms);
    println!("Your move: {}", mv);
    board.do_move(mv, player_color);

    send_command(writer, logger, &SendCommand::Move(mv))?;
    logger.log(board);

    match receive_command(reader, logger)? {
        RecvCommand::Ack {
            assigned_time_ms: updated,
        } => {
            *assigned_time_ms = updated;
            Ok(State::OpTurn(None))
        }
        RecvCommand::End {
            result,
            your_stone_count,
            opponent_stone_count,
            reason,
        } => Ok(State::EndGame {
            result,
            your_stone_count,
            opponent_stone_count,
            reason,
        }),
        r => Err(Error::Recv(r)),
    }
}

fn op_move(
    reader: &mut BufReader<&TcpStream>,
    logger: &mut Logger,
    board: &mut Board,
    player_color: Color,
    kihu: &mut Vec<Move>,
) -> Result<State> {
    match receive_command(reader, logger)? {
        RecvCommand::Move(m) => {
            println!("Opponent's move: {}", m);
            println!("{board}");
            board.do_move(m, player_color.opposite());
            print!("{}", board);
            logger.log(board);
            kihu.push(m);
            Ok(State::MyTurn(None))
        }
        RecvCommand::End {
            result,
            your_stone_count,
            opponent_stone_count,
            reason,
        } => Ok(State::EndGame {
            result,
            your_stone_count,
            opponent_stone_count,
            reason,
        }),
        r => Err(Error::Recv(r)),
    }
}

fn proc_end(
    result: Wl,
    your_stone_count: u32,
    opponent_stone_count: u32,
    reason: &str,
    player_name: &str,
    opponent_name: &str,
    player_color: Color,
) {
    let result_str = match result {
        Wl::Win => "You win!",
        Wl::Lose => "You lose!",
        Wl::Tie => "Draw",
    };
    println!("{result_str} ({your_stone_count} vs. {opponent_stone_count}) -- {reason}.",);
    println!(
        "Your name: {player_name} ({player_color})  Opponent name: {opponent_name} ({opponent_color}).",
        opponent_color = player_color.opposite()
    );
}

fn client(options: &MyOptions) -> Result<()> {
    println!("{:?}", options.socket_addr);
    let stream = TcpStream::connect(options.socket_addr)?;
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let mut logger = Logger::new(options);
    send_command(
        &mut writer,
        &mut logger,
        &(SendCommand::Open {
            player_name: &options.player,
        }),
    )?;

    let book_dict = use_book::initialize_book_dict();

    let mut state = State::WaitStart;
    let mut board = None;
    let mut assigned_time_ms = 0i32;
    let mut opponent_name = String::new();
    let mut player_color = Color::Black;
    let mut kihu: Vec<Move> = Vec::new();
    loop {
        match state {
            State::WaitStart => {
                state = wait_start(&mut reader, &mut logger)?;
            }
            State::MyTurn(Some(init_game)) => {
                assigned_time_ms = init_game.assigned_time_ms;
                opponent_name = init_game.opponent_name;
                board = Some(Board::new());
                state = State::MyTurn(None);
                player_color = Color::Black;
                kihu = Vec::new();
            }
            State::OpTurn(Some(init_game)) => {
                assigned_time_ms = init_game.assigned_time_ms;
                opponent_name = init_game.opponent_name;
                board = Some(Board::new());
                state = State::OpTurn(None);
                player_color = Color::White;
                kihu = Vec::new();
            }
            State::MyTurn(None) => {
                state = my_move(
                    &mut reader,
                    &mut writer,
                    &mut logger,
                    board.as_mut().expect("board must be initialized"),
                    player_color,
                    &mut assigned_time_ms,
                    &book_dict,
                    &mut kihu,
                )?;
            }
            State::OpTurn(None) => {
                state = op_move(
                    &mut reader,
                    &mut logger,
                    board.as_mut().expect("board must be initialized"),
                    player_color,
                    &mut kihu,
                )?;
            }
            State::EndGame {
                result,
                your_stone_count,
                opponent_stone_count,
                reason,
            } => {
                proc_end(
                    result,
                    your_stone_count,
                    opponent_stone_count,
                    &reason,
                    &options.player,
                    &opponent_name,
                    player_color,
                );
                state = State::WaitStart;
            }
            State::Exit { stat } => {
                print_scores(&mut logger, stat.into_iter());
                break;
            }
        }
    }
    Ok(())
}

fn main() {
    let options = parse_args();
    client(&options).unwrap_or_else(|e| {
        eprintln!("{e}");
    });
}
