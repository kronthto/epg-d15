#[macro_use]
extern crate lazy_static;

use crate::game::{D15Game, Point, PlayerState, Move};
use std::thread;
use std::process::exit;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash,Hasher};
use std::sync::{Mutex, MutexGuard};
use std::ops::{DerefMut};
use hash_hasher::{HashedSet,HashBuildHasher};

mod game;

// TODO: Impl other performance stuff: order?

lazy_static! {
    static ref CHECKED_PERMUTATIONS: Mutex<HashedSet<u64>> = {
        Mutex::new(HashedSet::with_capacity_and_hasher(1000000, HashBuildHasher::default()))
    };
}

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn main() {
    //let game = D15Game::new(50, Point { x: 2, y: 5 },  Point { x: 4, y: 6 },  Point { x: 0, y: 7 },  Point { x: 4, y: 5 },  Point { x: 3, y: 5 }, PlayerState::ARMOR); // att 7
   // let game = D15Game::new(55, Point { x: 3, y: 5 },  Point { x: 1, y: 7 },  Point { x: 7, y: 6 },  Point { x: 0, y: 4 },  Point { x: 2, y: 2 }, PlayerState::ARMOR); // att 5
 //   let game = D15Game::new(80, Point { x: 2, y: 5 },  Point { x: 4, y: 5 },  Point { x: 5, y: 3 },  Point { x: 7, y: 7 },  Point { x: 2, y: 2 }, PlayerState::ARMOR); // att 7 80
   // let game = D15Game::new(109, Point { x: 2, y: 5 },  Point { x: 3, y: 5 },  Point { x: 5, y: 4 },  Point { x: 7, y: 6 },  Point { x: 5, y: 5 }, PlayerState::ARMOR); // att 7 80
   // let game = D15Game::new(128, Point { x: 4, y: 5 },  Point { x: 6, y: 4 },  Point { x: 4, y: 6 },  Point { x: 3, y: 3 },  Point { x: 2, y: 4 }, PlayerState::SWORD); // att 2
   // let game = D15Game::new(130, Point { x: 4, y: 5 },  Point { x: 7, y: 1 },  Point { x: 2, y: 4 },  Point { x: 3, y: 0 },  Point { x: 6, y: 6 }, PlayerState::ARMOR); // att 4
  //  let game = D15Game::new(130, Point { x: 2, y: 5 },  Point { x: 4, y: 5 },  Point { x: 2, y: 4 },  Point { x: 5, y: 6 },  Point { x: 2, y: 3 }, PlayerState::ARMOR); // OMG
    let game = D15Game::new(130, Point { x: 5, y: 5 },  Point { x: 4, y: 5 },  Point { x: 2, y: 4 },  Point { x: 5, y: 6 },  Point { x: 2, y: 3 }, PlayerState::ARMOR); // OMG 5/5

    let mut game_switch = game.clone();
    let mut game_pass = game.clone();

    let handle1 = thread::spawn(move || {
        game_switch.do_move(&Move::SWITCH);
        let mut moves_done = vec![Move::SWITCH];
        solve(&game_switch, &mut moves_done);
    });
    let handle2 = thread::spawn(move || {
        game_pass.do_move(&Move::PASSTURN);
        let mut moves_done = vec![Move::PASSTURN];
        solve(&game_pass, &mut moves_done);
    });
    let handle3 = thread::spawn(move || {
        let moves = game.get_possible_moves();

        let moves_done : Vec<Move> = Vec::new();

        for move_oper in &moves {

            if *move_oper == Move::PASSTURN || *move_oper == Move::SWITCH {
                continue;
            }

            let mut new_game = game.clone();
            new_game.do_move(move_oper);
            let mut new_moves_done = moves_done.to_vec();
            new_moves_done.push(*move_oper);
            solve(&new_game, &mut new_moves_done);
        }
    });

    handle1.join();
    handle2.join();
    handle3.join();
}

fn solve(game: &D15Game, moves_done: &mut Vec<Move>) {
    if game.check_over_dead() {
        return;
    }
    if game.check_win() {
        println!("WIN!!!");
        print_result_moves(moves_done);
        exit(0);
    }

    let game_hash = calculate_hash(game);
    {
        let mut checked_perms_guard: MutexGuard<HashedSet<u64>> = CHECKED_PERMUTATIONS.lock().unwrap();
        let checked_perms = checked_perms_guard.deref_mut();
        if checked_perms.contains(&game_hash) {
            return;
        }
        checked_perms.insert(game_hash);
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

