use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::{stdin, stdout, Write};
use std::env;

use hash_hasher::{HashBuildHasher, HashedSet};

use crate::game::{D15Game, Move, PlayerState, Point, Color};

mod game;

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
         Color::YELLOW // TODO: Add param
     );


    let mut solver = Solver::new();
    solver.do_solve(&game);

    if solver.solve.is_none() {
        println!("UNSOLV");
        return;
    }
    print_result_moves(&solver.solve.unwrap());
}

struct Solver {
    besthp: i16,
    solve: Option<Vec<Move>>,
    checked_perms: HashedSet<u64>,
}
impl Solver {
    pub fn new() -> Solver {
        Solver {
            solve: None,
            besthp: 160,
            checked_perms: HashedSet::with_capacity_and_hasher(1000000, HashBuildHasher::default())
        }
    }

    pub fn do_solve(&mut self, game: &D15Game) {
        let mut possible_start_moves = game.get_possible_moves();

        let index_switch = possible_start_moves.iter().position(|x| *x == Move::SWITCH).unwrap();
        possible_start_moves.remove(index_switch);
        possible_start_moves.insert(0, Move::SWITCH);

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
            print_result_moves(moves_done);
            self.solve = Some(moves_done.to_vec());
            self.besthp = game.hp;
            println!();
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

