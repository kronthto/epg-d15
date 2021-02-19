#[macro_use]
extern crate lazy_static;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::{DerefMut, Deref};
use std::sync::{RwLock,  RwLockReadGuard, RwLockWriteGuard};
use std::thread;
use std::io::{stdin, stdout, Write};
use std::env;

use hash_hasher::{HashBuildHasher, HashedSet};

use crate::game::{D15Game, Move, PlayerState, Point, Color};
use std::cmp::min;

mod game;

lazy_static! {
    static ref CHECKED_PERMUTATIONS: RwLock<HashedSet<u64>> = {
        RwLock::new(HashedSet::with_capacity_and_hasher(1000000, HashBuildHasher::default()))
    };
}

static mut FINISHED: bool = false;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn ask_input(q: &str) -> String {
    print!("{}", q);
    let _=stdout().flush();
    let mut s = String::new();
    stdin().read_line(&mut s).expect("Did not enter a correct string");
    if let Some('\n')=s.chars().next_back() {
        s.pop();
    }
    if let Some('\r')=s.chars().next_back() {
        s.pop();
    }
    s
}

fn ask_playerstate(input: &String) -> PlayerState {
    match input.as_str() {
        "S" => PlayerState::SWORD,
        "A" => PlayerState::ARMOR,
        _ => panic!("PlayerState must be S or A")
    }
}

fn main() {
    let cpus = (num_cpus::get() / 2) + 1;

    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        panic!("1 arg expected")
    }

    let split = args[1].split("_");
    let res: Vec<String> = split.map(|s| s.to_string()).collect();

    if res.len() != 12 {
        panic!("Invalid input")
    }

    let hp : i16 = res[0].parse().unwrap();
    if hp > 200 || hp < 50 {
        panic!("Invalid start hp")
    }
    let players_state = ask_playerstate(&res[11]);

     let game = D15Game::new(
         hp,
         Point { x: res[1].parse().unwrap(), y: res[2].parse().unwrap() },
         Point { x: res[3].parse().unwrap(), y: res[4].parse().unwrap() },
         Point { x: res[5].parse().unwrap(), y: res[6].parse().unwrap() },
         Point { x: res[7].parse().unwrap(), y: res[8].parse().unwrap() },
         Point { x: res[9].parse().unwrap(), y: res[10].parse().unwrap() },
         players_state,
         Color::YELLOW
     );

    let mut possible_start_moves = game.get_possible_moves();
    let num_threads = min(possible_start_moves.len(), cpus) - 2;

    let index_switch = possible_start_moves.iter().position(|x| *x == Move::SWITCH).unwrap();
    possible_start_moves.remove(index_switch);
    possible_start_moves.insert(0, Move::SWITCH);

    // TODO: 1 or remaining in case too little, Switch auf jeden Fall nach oben / eig thread
    let mut threads: Vec<_> = (0..num_threads)
        .map(|i| {
            let mut tg = game.clone();
            let tm = *possible_start_moves.get(i).unwrap();
            thread::spawn(move || {
                println!("Spawned Thread {}", i);
                tg.do_move(&tm);
                let mut moves_done = vec![tm];
                solve(&tg, &mut moves_done);
            })
        })
        .collect();

    let mut remainder_moves = possible_start_moves.to_vec();
    remainder_moves.drain(0..num_threads);
    threads.push(   thread::spawn(move || {
        for move_oper in remainder_moves {
            let mut new_game = game.clone();
            new_game.do_move(&move_oper);
            let mut moves_done = vec![move_oper];
            solve(&new_game, &mut moves_done);
        }
    }));

    for t in threads {
        t.join().expect("Thread panicked");
    }
}

fn solve(game: &D15Game, moves_done: &mut Vec<Move>) {
    unsafe {
        if FINISHED {return;}
    }
    if game.check_over_dead() {
        return;
    }
    if game.check_win_2() {
        print_result_moves(moves_done);
        unsafe { FINISHED = true; }
        return;
    }

    let game_hash = calculate_hash(game);

    {
        let checked_perms_guard: RwLockReadGuard<HashedSet<u64>> = CHECKED_PERMUTATIONS.read().unwrap();
        let checked_perms = checked_perms_guard.deref();
        if checked_perms.contains(&game_hash) {
            return;
        }
    }
    {
        let mut checked_perms_guard_write: RwLockWriteGuard<HashedSet<u64>> = CHECKED_PERMUTATIONS.write().unwrap();
        let checked_perms_write = checked_perms_guard_write.deref_mut();
        checked_perms_write.insert(game_hash);
        /*
        let seen_boards = checked_perms_write.len();
        if seen_boards % 1000000 == 0 {
            println!("Seen {} boards", seen_boards);
        }
        */
    }

    let moves = game.get_possible_moves();

    for move_oper in &moves {
        let mut new_game = game.clone();
        let mut new_moves_done = moves_done.to_vec();
        new_game.do_move(move_oper);
        new_moves_done.push(*move_oper);
        solve(&new_game, &mut new_moves_done);
    }
}

fn print_result_moves(moves: &Vec<Move>) {
    for move_oper in moves {
        match move_oper {
            Move::PASSTURN => print!("PASSTURN_"),
            Move::LEFT => print!("LEFT_"),
            Move::RIGHT => print!("RIGHT_"),
            Move::UP => print!("UP_"),
            Move::DOWN => print!("DOWN_"),
            Move::DOG => print!("DOG_"),
            Move::CAT => print!("CAT_"),
            Move::DRAGON => print!("DRAGON_"),
            Move::SWITCH => print!("SWITCH_"),
        };
    }
}

