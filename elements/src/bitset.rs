use crate::Entity;

#[derive(Clone, Debug)]
pub struct BitSet {
    words: Vec<u64>,
}
impl BitSet {
    pub fn new_empty(initial_capacity: usize) -> Self {
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

    // pub fn contains(&self, e: Entity) -> bool {
    //     let w = (e as usize) / 64;
    //     if w >= self.words.len() {
    //         return false;
    //     }
    //     let b = (e as usize) % 64;
    //     (self.words[w] >> b) & 1 != 0
    // }

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

    pub fn collect_set(&self) -> Vec<Entity> {
        let mut result = Vec::new();
        for (i, &word) in self.words.iter().enumerate() {
            if word == 0 {
                continue;
            }
            for j in 0..64 {
                if (word >> j) & 1 != 0 {
                    result.push((i * 64 + j) as Entity);
                }
            }
        }
        result
    }

    // fn is_empty(&self) -> bool {
    //     self.words.iter().all(|&w| w == 0)
    // }
}
