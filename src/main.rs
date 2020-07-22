#[macro_use]
extern crate lazy_static;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::{DerefMut, Deref};
use std::sync::{RwLock,  RwLockReadGuard, RwLockWriteGuard};
use std::thread;
use std::io::{stdin, stdout, Write};

use hash_hasher::{HashBuildHasher, HashedSet};

use crate::game::{D15Game, Move, PlayerState, Point};
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

fn ask_playerstate() -> PlayerState {
    match ask_input("Playerstate (S or A): ").as_str() {
        "S" => PlayerState::SWORD,
        "A" => PlayerState::ARMOR,
        _ => panic!("PlayerState must be S or A")
    }
}

fn main() {
    // let game = D15Game::new(50, Point { x: 2, y: 5 },  Point { x: 4, y: 6 },  Point { x: 0, y: 7 },  Point { x: 4, y: 5 },  Point { x: 3, y: 5 }, PlayerState::ARMOR); // att 7
   // let game = D15Game::new(55, Point { x: 3, y: 5 },  Point { x: 1, y: 7 },  Point { x: 7, y: 6 },  Point { x: 0, y: 4 },  Point { x: 2, y: 2 }, PlayerState::ARMOR); // att 5
    // let game = D15Game::new(80, Point { x: 2, y: 5 },  Point { x: 4, y: 5 },  Point { x: 5, y: 3 },  Point { x: 7, y: 7 },  Point { x: 2, y: 2 }, PlayerState::ARMOR); // att 7 80
    // let game = D15Game::new(109, Point { x: 2, y: 5 },  Point { x: 3, y: 5 },  Point { x: 5, y: 4 },  Point { x: 7, y: 6 },  Point { x: 5, y: 5 }, PlayerState::ARMOR); // att 7 80
    // let game = D15Game::new(128, Point { x: 4, y: 5 },  Point { x: 6, y: 4 },  Point { x: 4, y: 6 },  Point { x: 3, y: 3 },  Point { x: 2, y: 4 }, PlayerState::SWORD); // att 2
    // let game = D15Game::new(130, Point { x: 4, y: 5 },  Point { x: 7, y: 1 },  Point { x: 2, y: 4 },  Point { x: 3, y: 0 },  Point { x: 6, y: 6 }, PlayerState::ARMOR); // att 4
    //let game = D15Game::new(130, Point { x: 2, y: 5 },  Point { x: 4, y: 5 },  Point { x: 2, y: 4 },  Point { x: 5, y: 6 },  Point { x: 2, y: 3 }, PlayerState::ARMOR); // OMG
    // let game = D15Game::new(130, Point { x: 5, y: 5 }, Point { x: 4, y: 5 }, Point { x: 2, y: 4 }, Point { x: 5, y: 6 }, Point { x: 2, y: 3 }, PlayerState::ARMOR); // OMG 5/5

    let cpus = (num_cpus::get() / 2) + 1;

    let players_state = ask_playerstate();

     let game = D15Game::new(
         ask_input("Enter HP: ").parse().unwrap(),
         Point { x: ask_input("Enter Boss X: ").parse().unwrap(), y: ask_input("Enter Boss Y: ").parse().unwrap() },
         Point { x: ask_input("Enter Player X: ").parse().unwrap(), y: ask_input("Enter Player Y: ").parse().unwrap() },
         Point { x: ask_input("Enter Cat X: ").parse().unwrap(), y: ask_input("Enter Cat Y: ").parse().unwrap() },
         Point { x: ask_input("Enter Dog X: ").parse().unwrap(), y: ask_input("Enter Dog Y: ").parse().unwrap() },
         Point { x: ask_input("Enter Dragon X: ").parse().unwrap(), y: ask_input("Enter Dragon Y: ").parse().unwrap() },
         players_state
     );

    let mut possible_start_moves = game.get_possible_moves();
    let num_threads = min(possible_start_moves.len(), cpus) - 2;

    println!("-> Press Ctrl+C to exit");

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
        println!("Spawned remainder Thread");
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

    println!("Done / all checked");

    let _ = stdin().read_line(&mut String::new()).unwrap(); // keep console open
}

fn solve(game: &D15Game, moves_done: &mut Vec<Move>) {
    unsafe {
        if FINISHED {return;}
    }
    if game.check_over_dead() {
        return;
    }
    if game.check_win() {
        println!("WIN!!!");
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
        let seen_boards = checked_perms_write.len();
        if seen_boards % 1000000 == 0 {
            println!("Seen {} boards", seen_boards);
        }
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
            Move::PASSTURN => println!("PASSTURN"),
            Move::LEFT => println!("LEFT"),
            Move::RIGHT => println!("RIGHT"),
            Move::UP => println!("UP"),
            Move::DOWN => println!("DOWN"),
            Move::DOG => println!("DOG"),
            Move::CAT => println!("CAT"),
            Move::DRAGON => println!("DRAGON"),
            Move::SWITCH => println!("SWITCH"),
        };
    }
}

