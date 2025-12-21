#![allow(unused_imports, dead_code)]

use std::{
    collections::HashMap,
    f32::consts::E,
    hash::{BuildHasher, Hasher},
    process::id,
};

use crate::{bitboard::consts::*, board::Slot, game::Game, zobrist::GLOBAL_ZOBRIST};

// TODO: impl custom hashmap for games using this zobrist impl
// https://medium.com/better-programming/implementing-a-hashmap-in-rust-35d055b5ac2b
// https://www.chessprogramming.org/Transposition_Table
// https://www.chessprogramming.org/Perft

#[derive(Clone, Debug)]
struct Entry {
    key: (Option<Game>, Slot),
    value: i32,
}

impl Entry {
    fn new() -> Self {
        Self {
            key: (None, Slot::Disabled),
            value: 0,
        }
    }

    fn new_tomb() -> Self {
        Self {
            key: (None, Slot::Disabled),
            value: 4,
        }
    }

    fn is_tomb(&self) -> bool {
        self.key.0.is_none() && self.value == 4
    }
}

pub struct Table {
    entries: Vec<Entry>,
    count: usize,
}

impl Table {
    pub fn new() -> Self {
        Self {
            entries: vec![Entry::new(); 1000],
            count: 0,
        }
    }

    fn resize(&mut self) {
        let mut old_vec = vec![Entry::new(); self.entries.len() * 2];

        std::mem::swap(&mut old_vec, &mut self.entries);

        self.count = 0;
        for entry in old_vec.into_iter().filter(|e| e.key.0.is_some()) {
            self.insert((entry.key.0.unwrap(), entry.key.1), entry.value);
        }
    }

    pub fn insert(&mut self, key: (Game, Slot), value: i32) -> bool {
        // 3 / 4 = 0.75, our chosen load factor, cant just do x0.75 since
        // they're integers
        if (self.count + 1) as f64 > self.entries.len() as f64 * 0.7 {
            self.resize();
        }

        let entry = self.find_entry_mut(&key.0, key.1);
        let is_replacing = entry.key.0.is_some();
        let is_new = entry.key.0.is_none() && !entry.is_tomb();

        *entry = Entry {
            key: (Some(key.0), key.1),
            value,
        };

        if is_new {
            self.count += 1;
        }

        !is_replacing
    }

    #[inline]
    pub fn get(&self, game: &Game, slot: Slot) -> Option<i32> {
        let entry = self.find_entry(game, slot);

        if entry.key.0.is_some() {
            Some(entry.value)
        } else {
            None
        }
    }

    pub fn delete(&mut self, key: (&Game, Slot)) -> Option<(Game, i32)> {
        let ent = self.find_entry_mut(key.0, key.1);

        if ent.key.0.is_some() {
            let old = std::mem::replace(ent, Entry::new_tomb());

            Some((old.key.0.unwrap(), old.value))
        } else {
            None
        }
    }

    #[inline]
    fn find_entry_mut<'b, 'a: 'b>(&'a mut self, game: &Game, slot: Slot) -> &'b mut Entry {
        let idx = self.find_index(game, slot);
        &mut self.entries[idx]
    }

    #[inline]
    fn find_entry<'b, 'a: 'b>(&'a self, game: &Game, slot: Slot) -> &'b Entry {
        let idx = self.find_index(game, slot);
        &self.entries[idx]
    }

    //  TODO: look into evicting old/worse entries
    fn find_index(&self, game: &Game, slot: Slot) -> usize {
        let h_cap = self.entries.len();
        let mut index = GLOBAL_ZOBRIST.hash(game, slot) as usize % h_cap;

        let mut tomb_idx = None;
        loop {
            let ent = &self.entries[index];
            let (ref k_g, k_s) = ent.key;

            if (k_g.as_ref() == Some(game) && k_s == slot) || (k_g.is_none() && !ent.is_tomb()) {
                let idx = if let Some(t_index) = tomb_idx {
                    t_index
                } else {
                    index
                };

                return idx;
            } else if ent.is_tomb() {
                tomb_idx = Some(index);
            }

            index = (index + 1) % h_cap;
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use std::{array, collections::HashSet};

    use rand::random;

    use crate::{board::Slot, counting::score_game, game::Game, table::Table};

    #[test]
    fn insert_get_50k() {
        let mut table = Table::new();
        let games: Vec<Game> = (0..50_000)
            .map(|_| Game::random_seedless(10))
            .collect::<HashSet<Game>>()
            .into_iter()
            .collect();

        let arr: Vec<(Game, Slot, i32)> = games
            .into_iter()
            .map(|g| {
                let turn = if random() { Slot::X } else { Slot::O };
                let score = score_game(&g, turn);

                (g, turn, score)
            })
            .collect();

        for el in &arr {
            assert!(table.insert((el.0.clone(), el.1), el.2));
        }

        for el in arr {
            assert_eq!(table.get(&el.0, el.1), Some(el.2));
        }
    }
}
