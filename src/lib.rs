use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use hash_hasher::{HashBuildHasher, HashedSet};

use wasm_bindgen::prelude::*;

use crate::game::{D15Game, Move, PlayerState, Point, Color};

mod game;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn ask_playerstate(input: &String) -> PlayerState {
    match input.as_str() {
        "S" => PlayerState::SWORD,
        "A" => PlayerState::ARMOR,
        _ => panic!("PlayerState must be S or A")
    }
}
fn ask_color(input: &String) -> Color { // TODO: YELLOW if 200 and blocked / or any <130
    match input.as_str() {
        "Y" => Color::YELLOW,
        "G" => Color::GREEN,
        "R" => Color::RED,
        "B" => Color::BLUE,
        _ => panic!("Invalid color")
    }
}

#[wasm_bindgen]
pub fn solve_d15_from_magicstr(magic_string : String) -> String {
    let split = magic_string.split("_");
    let res: Vec<String> = split.map(|s| s.to_string()).collect();

    if res.len() != 13 {
        panic!("Invalid input")
    }

    let hp : i16 = res[0].parse().unwrap();
    if hp > 200 || hp < 50 {
        panic!("Invalid start hp")
    }
    let players_state = ask_playerstate(&res[11]);
    let color = ask_color(&res[12]);

     let game = D15Game::new(
         hp,
         Point { x: res[1].parse().unwrap(), y: res[2].parse().unwrap() },
         Point { x: res[3].parse().unwrap(), y: res[4].parse().unwrap() },
         Point { x: res[5].parse().unwrap(), y: res[6].parse().unwrap() },
         Point { x: res[7].parse().unwrap(), y: res[8].parse().unwrap() },
         Point { x: res[9].parse().unwrap(), y: res[10].parse().unwrap() },
         players_state,
         color
     );

    let mut solver = Solver::new();
    solver.do_solve(&game);

    if solver.solve.is_none() {
        return "Could not solve - try again ~20 hp down".to_string();
    }
    return print_result_moves(&solver.solve.unwrap());
}

struct Solver {
    besthp: i16,
    solve: Option<Vec<Move>>,
    checked_perms: HashedSet<u64>,
    search_best: bool,
}
impl Solver {
    pub fn new() -> Solver {
        Solver {
            solve: None,
            besthp: 0,
            checked_perms: HashedSet::with_capacity_and_hasher(1000000, HashBuildHasher::default()),
            search_best: false
        }
    }

    pub fn do_solve(&mut self, game: &D15Game) {
        let mut possible_start_moves = game.get_possible_moves();

        if game.hp > 159 {
            self.search_best = true;
            self.besthp = game.hp-40;
        } else if game.hp > 128 {
            self.besthp = 100;
        } else {
            if game.get_boss_x() == 5 {
                self.besthp = 54;
            } else {
                self.besthp = 47;
            }
        }
        if game.hp <= 60 {
            self.besthp = 35;
        }

        if game.playerstate == PlayerState::SWORD {
            let index_switch = possible_start_moves.iter().position(|x| *x == Move::SWITCH).unwrap();
            possible_start_moves.remove(index_switch);
            possible_start_moves.insert(0, Move::SWITCH);
        }

        for move_oper in possible_start_moves {
            let mut new_game = game.clone();
            new_game.do_move(&move_oper);
            let moves_done = vec![move_oper];
            self.solve(&new_game, &moves_done);
        }
    }

    fn solve(&mut self, game: &D15Game, moves_done: &Vec<Move>) {
            if game.hp <= self.besthp {
                return;
            }

        if game.check_win_2() {
            //print_result_moves(moves_done);
            self.solve = Some(moves_done.to_vec());
            self.besthp = game.hp;
            if !self.search_best {
                self.besthp = 201;
            }
            //println!();
            return;
        }

        let game_hash = calculate_hash(game);

        if self.checked_perms.contains(&game_hash) {
            return;
        }
        self.checked_perms.insert(game_hash);
            /*
            let seen_boards = checked_perms_write.len();
            if seen_boards % 1000000 == 0 {
                println!("Seen {} boards", seen_boards);
            }
            */

        let moves = game.get_possible_moves();

        for move_oper in &moves {
            let mut new_game = game.clone();
            let mut new_moves_done = moves_done.to_vec();
            new_game.do_move(move_oper);
            new_moves_done.push(*move_oper);
            self.solve(&new_game, &new_moves_done);
        }
    }
}

fn print_result_moves(moves: &Vec<Move>) -> String {
    let mut parts : Vec<&str> = vec![];
    for move_oper in moves {
        match move_oper {
            Move::PASSTURN => parts.push("PASSTURN_"),
            Move::LEFT => parts.push("LEFT_"),
            Move::RIGHT => parts.push("RIGHT_"),
            Move::UP => parts.push("UP_"),
            Move::DOWN => parts.push("DOWN_"),
            Move::DOG => parts.push("DOG_"),
            Move::CAT => parts.push("CAT_"),
            Move::DRAGON => parts.push("DRAGON_"),
            Move::SWITCH => parts.push("SWITCH_"),
        };
    }
    parts.join("")
}

