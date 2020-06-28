use std::hash::Hash;
use std::mem::swap;

use crate::game::Entity::{BOSS, DOG, DRAGON, PLAYER};

pub const ROOM_MAX_X: i8 = 7;
pub const ROOM_MAX_Y: i8 = 7;

const PATTERN_1_LIMIT: i16 = 130;
const PATTERN_2_LIMIT: i16 = 100;
// TODO: This (P2 limit) changed
const PATTERN_3_LIMIT: i16 = 60;

#[derive(Hash, Eq)]
pub struct Point {
    pub(crate) x: i8,
    pub(crate) y: i8,
}

impl Clone for Point {
    fn clone(&self) -> Point {
        Point { x: self.x, y: self.y }
    }
}

impl PartialEq for Point {
    fn eq(&self, other: &Point) -> bool {
        self.x == other.x && self.y == other.y
    }
}

impl Point {
    fn mirror_by(&self, reference_point: &Point) -> Point {
        let xdiff = reference_point.x - self.x;
        let ydiff = reference_point.y - self.y;
        return Point { x: self.x + (2 * xdiff), y: self.y + (2 * ydiff) };
    }

    fn mirror_by_roomcenter(&self) -> Point {
        Point { x: -self.x + ROOM_MAX_X, y: -self.y + ROOM_MAX_Y }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub enum PlayerState {
    SWORD,
    ARMOR,
}

#[derive(PartialEq, Copy, Clone)]
pub enum Move {
    LEFT,
    RIGHT,
    UP,
    DOWN,
    DOG,
    CAT,
    DRAGON,
    SWITCH,
    PASSTURN,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Entity {
    PLAYER,
    BOSS,
    CAT,
    DOG,
    DRAGON,
}

const ENTITIES: [Entity; 5] = [PLAYER, BOSS, Entity::CAT, DOG, DRAGON];

#[derive(Eq, PartialEq, Hash)]
pub struct D15Game {
    hp: i16,
    player: Point,
    boss: Point,
    cat: Point,
    dog: Point,
    dragon: Point,
    playerstate: PlayerState,
}

impl D15Game {
    fn is_point_blocked_for(&self, entity: Entity, target_point: &Point) -> bool
    {
        ENTITIES.iter().filter(|&&x| x != entity).any(|&eachentity| self.get_entity_position(eachentity) == target_point)
    }

    pub fn get_entity_position(&self, entity: Entity) -> &Point {
        match entity {
            PLAYER => &self.player,
            BOSS => &self.boss,
            Entity::CAT => &self.cat,
            DOG => &self.dog,
            DRAGON => &self.dragon,
        }
    }

    fn can_move_to(&self, target_point: &Point, entity: Entity) -> bool {
        if target_point.x > ROOM_MAX_X || target_point.x < 0 {
            return false;
        }
        if target_point.y > ROOM_MAX_Y || target_point.y < 0 {
            return false;
        }
        return !self.is_point_blocked_for(entity, target_point);
    }

    fn find_move_until(&self, for_entity: Entity, step_x: i8, step_y: i8, steps: u8) -> Point {
        let start = self.get_entity_position(for_entity);
        let mut ok = start.clone();
        let mut x_carry = start.x;
        let mut y_carry = start.y;
        for _ in 0..steps {
            x_carry += step_x;
            y_carry += step_y;
            let point_to_check = Point { x: x_carry, y: y_carry };
            if self.can_move_to(&point_to_check, for_entity) {
                ok = point_to_check;
            } else {
                break;
            }
        };
        return ok;
    }

    pub fn check_win(&self) -> bool {
        [
            Point {
                x: self.boss.x + 1,
                y: self.boss.y,
            },
            Point {
                x: self.boss.x - 1,
                y: self.boss.y,
            },
            Point {
                x: self.boss.x,
                y: self.boss.y + 1,
            },
            Point {
                x: self.boss.x,
                y: self.boss.y - 1,
            }
        ].iter().all(|required_point| ENTITIES.iter().any(|entity| self.get_entity_position(*entity) == required_point))
    }

    pub fn check_over_dead(&self) -> bool {
        return self.hp <= 0;
    }

    fn get_moveamount(&self) -> i8 {
        match self.playerstate {
            PlayerState::SWORD => 2,
            PlayerState::ARMOR => 1,
        }
    }

    pub fn get_possible_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(9);

        let move_amount = self.get_moveamount();

        if self.hp > PATTERN_3_LIMIT {
            moves.push(Move::PASSTURN);
        }

        if self.can_move_to(&Point { x: self.player.x - move_amount, y: self.player.y }, Entity::PLAYER) {
            moves.push(Move::LEFT);
        }
        if self.can_move_to(&Point { x: self.player.x + move_amount, y: self.player.y }, Entity::PLAYER) {
            moves.push(Move::RIGHT);
        }
        if self.can_move_to(&Point { x: self.player.x, y: self.player.y - move_amount }, Entity::PLAYER) {
            moves.push(Move::DOWN);
        }
        if self.can_move_to(&Point { x: self.player.x, y: self.player.y + move_amount }, Entity::PLAYER) {
            moves.push(Move::UP);
        }

        moves.push(Move::DOG);
        moves.push(Move::CAT);
        moves.push(Move::DRAGON);
        moves.push(Move::SWITCH);

        return moves;
    }

    pub fn do_move(&mut self, direction: &Move) {
        match direction {
            Move::SWITCH => match self.playerstate {
                PlayerState::SWORD => self.playerstate = PlayerState::ARMOR,
                PlayerState::ARMOR => self.playerstate = PlayerState::SWORD,
            },
            Move::PASSTURN => self.hp -= 25,
            Move::UP => self.player = Point { x: self.player.x, y: self.player.y + self.get_moveamount() },
            Move::DOWN => self.player = Point { x: self.player.x, y: self.player.y - self.get_moveamount() },
            Move::LEFT => self.player = Point { x: self.player.x - self.get_moveamount(), y: self.player.y },
            Move::RIGHT => self.player = Point { x: self.player.x + self.get_moveamount(), y: self.player.y },
            Move::DOG => swap(&mut self.player, &mut self.dog),
            Move::CAT => swap(&mut self.player, &mut self.cat),
            Move::DRAGON => swap(&mut self.player, &mut self.dragon),
        };

        if *direction != Move::PASSTURN {
            self.dog_move();
            self.cat_move();
            self.dragon_move();
        }

        self.hp -= match self.playerstate {
            PlayerState::SWORD => 4,
            PlayerState::ARMOR => 2,
        };
    }

    fn dog_move(&mut self) {
        if self.hp > PATTERN_1_LIMIT {
            // todo: find...
        } else if self.hp > PATTERN_2_LIMIT {
            swap(&mut self.dog, &mut self.cat);
        } else if self.hp > PATTERN_3_LIMIT {
            self.dog = self.find_move_until(Entity::DOG, 0, 1, 3);
        } else {
            swap(&mut self.dog, &mut self.dragon);
        }
    }
    fn cat_move(&mut self) {
        if self.hp > PATTERN_1_LIMIT {
            // move 1 towards dragon
            let diff_x = self.dragon.x - self.cat.x;
            let diff_y = self.dragon.y - self.cat.y;
            let cat_move: Point;
            if diff_x.abs() > diff_y.abs() {
                cat_move = Point { x: self.cat.x + diff_x.signum(), y: self.cat.y };
            } else {
                cat_move = Point { x: self.cat.x, y: self.cat.y + diff_y.signum() };
            }
            if self.can_move_to(&cat_move, Entity::CAT) {
                self.cat = cat_move;
            }
        } else if self.hp > PATTERN_2_LIMIT {
            self.cat = self.find_move_until(Entity::CAT, 0, -1, 3);
        } else if self.hp > PATTERN_3_LIMIT {
            // move 1 away from player
            let diff_x = self.cat.x - self.player.x;
            let diff_y = self.cat.y - self.player.y;
            let cat_move: Point;
            if diff_x.abs() >= diff_y.abs() {
                cat_move = Point { x: self.cat.x + diff_x.signum(), y: self.cat.y };
            } else {
                cat_move = Point { x: self.cat.x, y: self.cat.y + diff_y.signum() };
            }
            if self.can_move_to(&cat_move, Entity::CAT) {
                self.cat = cat_move;
            }
        } else {
            self.cat = self.find_move_until(Entity::CAT, 1, 0, 3);
        }
    }
    fn dragon_move(&mut self) {
        if self.hp > PATTERN_1_LIMIT {
            let mirror = self.dragon.mirror_by(&self.player);
            if self.can_move_to(&mirror, Entity::DRAGON) {
                self.dragon = mirror;
            }
        } else if self.hp > PATTERN_2_LIMIT {
            // nothing
        } else if self.hp > PATTERN_3_LIMIT {
            let mirror = self.dragon.mirror_by_roomcenter();
            if self.can_move_to(&mirror, Entity::DRAGON) {
                self.dragon = mirror;
            }
        } else {
            self.dragon = self.find_move_until(Entity::DRAGON, -1, 0, 3);
        }
    }

    pub fn new(hp: i16, boss: Point, player: Point, cat: Point, dog: Point, dragon: Point, playerstate: PlayerState) -> D15Game {
        D15Game {
            hp,
            player,
            boss,
            cat,
            dog,
            dragon,
            playerstate,
        }
    }
}

impl Clone for D15Game {
    fn clone(&self) -> D15Game {
        D15Game {
            hp: self.hp,
            player: self.player.clone(),
            boss: self.boss.clone(),
            cat: self.cat.clone(),
            dog: self.dog.clone(),
            dragon: self.dragon.clone(),
            playerstate: self.playerstate,
        }
    }
}
