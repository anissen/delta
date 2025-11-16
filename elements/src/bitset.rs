use crate::Entity;

#[derive(Clone, Debug)]
pub struct BitSet {
    words: Vec<u64>,
}
impl BitSet {
    pub fn new(initial_capacity: usize) -> Self {
        let mut bitset = BitSet { words: Vec::new() };
        bitset.ensure_capacity(initial_capacity as Entity);
        bitset
    }

    fn ensure_capacity(&mut self, entity: Entity) {
        let word_index = (entity as usize) / 64;
        if word_index >= self.words.len() {
            self.words.resize(word_index + 1, 0);
        }
    }

    pub fn set(&mut self, e: Entity) {
        self.ensure_capacity(e);
        let w = (e as usize) / 64;
        let b = (e as usize) % 64;
        self.words[w] |= 1u64 << b;
    }

    pub fn unset(&mut self, e: Entity) {
        let w = (e as usize) / 64;
        if w < self.words.len() {
            let b = (e as usize) % 64;
            self.words[w] &= !(1u64 << b);
        }
    }

    fn contains(&self, e: Entity) -> bool {
        let w = (e as usize) / 64;
        if w >= self.words.len() {
            return false;
        }
        let b = (e as usize) % 64;
        (self.words[w] >> b) & 1 != 0
    }

    pub fn intersect_with(&mut self, other: &BitSet) {
        let min_words = self.words.len().min(other.words.len());
        self.words.resize(min_words, 0);
        for i in 0..min_words {
            self.words[i] &= other.words[i];
        }
    }

    pub fn disjoint_with(&mut self, other: &BitSet) {
        let min_words = self.words.len().min(other.words.len());
        self.words.resize(min_words, 0);
        for i in 0..min_words {
            self.words[i] &= !other.words[i];
        }
    }

    fn is_empty(&self) -> bool {
        self.words.iter().all(|&w| w == 0)
    }

    /// Iterate entity ids present in the bitset.
    pub fn iter_ids(&self) -> BitSetIter<'_> {
        BitSetIter {
            words: &self.words,
            idx: 0,
            cur: 0,
        }
    }
}

pub struct BitSetIter<'a> {
    words: &'a [u64],
    idx: usize,
    cur: u64,
}
impl<'a> Iterator for BitSetIter<'a> {
    type Item = Entity;
    fn next(&mut self) -> Option<Self::Item> {
        while self.cur == 0 {
            if self.idx >= self.words.len() {
                return None;
            }
            self.cur = self.words[self.idx];
            self.idx += 1;
        }
        let tz = self.cur.trailing_zeros() as usize;
        self.cur &= !(1u64 << tz);
        let entity = ((self.idx - 1) * 64 + tz) as Entity;
        Some(entity)
    }
}
