#![allow(unused_imports, dead_code)]

use std::{
    collections::HashMap,
    hash::{BuildHasher, Hasher},
};

use crate::{bitboard::consts::*, board::Slot, game::Game};

// TODO: impl custom hashmap for games using this zobrist impl
// https://medium.com/better-programming/implementing-a-hashmap-in-rust-35d055b5ac2b
// https://www.chessprogramming.org/Transposition_Table
// https://www.chessprogramming.org/Perft

#[derive(Clone)]
struct TTEntry {
    key: Option<Game>,
    turn: Slot,
    val: i32,
}

struct Table {
    entries: Vec<TTEntry>,
    count: usize,
    zobr: ZobristBuilder,
}

impl Table {
    fn new() -> Self {
        Table {
            entries: vec![
                TTEntry {
                    key: None,
                    val: 0,
                    turn: Slot::Disabled
                };
                150
            ],
            count: 0,
            zobr: ZobristBuilder::new(),
        }
    }

    // TODO: take depth into account?
    pub fn insert(&mut self, game: Game, turn: Slot, score: i32) -> bool {
        if self.count + 1 > self.entries.capacity() * 7 / 10 {
            let ovec = std::mem::replace(
                &mut self.entries,
                vec![
                    TTEntry {
                        key: None,
                        val: 0,
                        turn: Slot::Disabled
                    };
                    self.count * 2
                ],
            );

            for ent in ovec {
                if let Some(ref game) = ent.key {
                    let dest = self.find_entry_mut(game, turn);
                    *dest = ent
                }
            }
        }

        let entry = self.find_entry_mut(&game, turn);
        let is_new = entry.key.is_none();

        entry.key = Some(game);
        entry.val = score;
        entry.turn = turn;

        if is_new {
            self.count += 1;
        }

        is_new
    }

    pub fn delete(&mut self, game: &Game, turn: Slot) -> Option<i32> {
        if self.count == 0 {
            return None;
        }

        let entry = self.find_entry(game, turn);
        entry.key.as_ref()?;

        todo!();
    }

    pub fn get(&self, game: &Game, turn: Slot) -> Option<i32> {
        if self.count == 0 {
            return None;
        }

        let entry = self.find_entry(game, turn);
        entry.key.as_ref()?;

        Some(entry.val)
    }

    fn find_entry(&self, game: &Game, turn: Slot) -> &TTEntry {
        let cap = self.entries.capacity() as u64;
        let mut idx = self.zobr.hash(game, turn) % cap;

        loop {
            let entry = &self.entries[idx as usize];

            if entry.key.is_none() || entry.key.as_ref() == Some(game) {
                return &self.entries[idx as usize];
            }

            idx = (idx + 1) % cap;
        }
    }

    fn find_entry_mut(&mut self, game: &Game, turn: Slot) -> &mut TTEntry {
        let cap = self.entries.capacity() as u64;
        let mut idx = self.zobr.hash(game, turn) % cap;

        loop {
            let entry = &self.entries[idx as usize];

            if entry.key.is_none() || entry.key.as_ref() == Some(game) {
                return &mut self.entries[idx as usize];
            }

            idx = (idx + 1) % cap;
        }
    }
}

struct ZobristBuilder {
    pieces_x: [[u64; 9]; 9],
    pieces_o: [[u64; 9]; 9],
    active: [u64; 10],
    x_turn: u64,
}

impl ZobristBuilder {
    fn new() -> Self {
        // {x, o} * 81 squares
        let pieces_x: [[u64; 9]; 9] = rand::random();
        let pieces_o: [[u64; 9]; 9] = rand::random();
        let active: [u64; 10] = rand::random();
        let x_turn: u64 = rand::random();

        ZobristBuilder {
            pieces_x,
            pieces_o,
            active,
            x_turn,
        }
    }

    fn hash(&self, game: &Game, turn: Slot) -> u64 {
        let mut hash = 0;

        // TODO: SIMD
        for (i, brd) in game.boards.iter().enumerate() {
            let mut bd = brd.0 & X_MASK;

            while bd != 0 {
                let ix = bd.trailing_zeros();
                hash ^= self.pieces_x[i][ix as usize];

                bd &= bd - 1;
            }
        }

        for (i, brd) in game.boards.iter().enumerate() {
            let mut bd = (brd.0 & O_MASK) >> O_OFFS;

            while bd != 0 {
                let ix = bd.trailing_zeros();
                hash ^= self.pieces_o[i][ix as usize];

                bd &= bd - 1;
            }
        }

        if turn == Slot::X {
            hash ^= self.x_turn;
        }

        hash ^= self.active[game.active];

        hash
    }
}

#[cfg(test)]
mod test {
    use crate::{board::Slot, game::Game};

    use super::Table;

    #[test]
    fn create_and_get_5000() {
        let mut table = Table::new();

        for times in 1..=5 {
            for _ in 0..1000 {
                let g = Game::random_seedless(times);
                let fake_score = rand::random();
                let turn = if rand::random() { Slot::X } else { Slot::O };

                table.insert(g.clone(), turn, fake_score);

                assert_eq!(fake_score, table.get(&g, turn).unwrap())
            }
        }
    }

    #[test]
    fn overwrite() {
        let mut table = Table::new();
        let game = Game::random_seedless(18);

        assert!(table.insert(game.clone(), Slot::X, 100));
        assert!(!table.insert(game.clone(), Slot::X, 100));
        assert!(!table.insert(game.clone(), Slot::X, 200));
        assert_eq!(table.get(&game, Slot::X), Some(200));
    }
}
