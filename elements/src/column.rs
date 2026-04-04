use crate::ComponentId;
use crate::ComponentLayout;
use crate::Entity;
use crate::bitset::BitSet;

#[derive(Debug)]
pub struct Column {
    pub id: ComponentId, // TODO(anissen): Needs to be encoded in World instead
    pub layout: ComponentLayout,
    dense: Vec<u8>,
    entities: Vec<Entity>,
    sparse: Vec<usize>,
    pub bitset: BitSet,
}

impl Column {
    pub fn new(
        component_id: ComponentId,
        layout: ComponentLayout,
        initial_capacity: usize,
    ) -> Self {
        let size = layout.size;
        Self {
            id: component_id,
            layout,
            dense: vec![0; initial_capacity * size], // allow zero-size components
            entities: Vec::with_capacity(initial_capacity),
            sparse: vec![usize::MAX; initial_capacity],
            bitset: BitSet::new_empty(initial_capacity),
        }
    }

    fn ensure_entity_capacity(&mut self, entity: Entity) {
        if entity as usize >= self.sparse.len() {
            self.sparse.resize(entity as usize + 1, usize::MAX); // TODO(anissen): Use a different magic constant
        }
    }

    fn ensure_dense_capacity(&mut self) {
        if self.layout.size == 0 {
            return; // marker component: no storage needed
        }

        let needed = (self.entities.len() + 1) * self.layout.size;
        if needed > self.dense.len() {
            let new_capacity = (self.dense.len().max(self.layout.size) * 2).max(needed);
            self.dense.resize(new_capacity, 0);
        }
    }

    pub fn insert(&mut self, entity: Entity, value_bytes: &[u8]) {
        self.ensure_entity_capacity(entity);

        let idx = self.sparse[entity as usize];

        if idx == usize::MAX {
            // Insert component
            self.ensure_dense_capacity();
            let new_index = self.entities.len();
            self.entities.push(entity);
            self.sparse[entity as usize] = new_index;

            if self.layout.size > 0 {
                let start = new_index * self.layout.size;
                let end = start + self.layout.size;
                self.dense[start..end].copy_from_slice(value_bytes);
            }
            self.bitset.set(entity);
        } else {
            // Replace component
            if self.layout.size > 0 {
                let start = idx * self.layout.size;
                let end = start + self.layout.size;
                self.dense[start..end].copy_from_slice(value_bytes);
            }
        }
    }

    // pub fn has(&self, entity: Entity) -> bool {
    //     self.bitset.contains(entity)
    // }

    pub fn get(&self, entity: Entity) -> Option<&[u8]> {
        let idx = self.sparse.get(entity as usize)?;
        if *idx == usize::MAX {
            return None;
        }
        if self.layout.size == 0 {
            return Some(&[]);
        }
        let start = idx * self.layout.size;
        let end = start + self.layout.size;
        Some(&self.dense[start..end])
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut [u8]> {
        let idx = self.sparse.get(entity as usize)?;
        if *idx == usize::MAX {
            return None;
        }
        if self.layout.size == 0 {
            return Some(&mut []);
        }
        let start = idx * self.layout.size;
        let end = start + self.layout.size;
        Some(&mut self.dense[start..end])
    }

    pub fn get_two_mut(
        &mut self,
        entity1: Entity,
        entity2: Entity,
    ) -> Option<(&mut [u8], &mut [u8])> {
        assert!(entity1 != entity2);
        let idx1 = self.sparse.get(entity1 as usize)?;
        if *idx1 == usize::MAX {
            return None;
        }
        let idx2 = self.sparse.get(entity2 as usize)?;
        if *idx2 == usize::MAX {
            return None;
        }
        if self.layout.size == 0 {
            return Some((&mut [], &mut []));
        }
        let start1 = idx1 * self.layout.size;
        let end1 = start1 + self.layout.size;
        let start2 = idx2 * self.layout.size;
        let end2 = start2 + self.layout.size;
        // dbg!(start1, end1);
        // dbg!(start2, end2);
        if start1 < start2 {
            assert!(end1 <= start2);
            let (left, right) = self.dense.split_at_mut(start2 as usize);
            Some((&mut left[start1..end1], &mut right[0..(end2 - start2)]))
        } else {
            assert!(end2 <= start1);
            let (left, right) = self.dense.split_at_mut(start1 as usize);
            Some((&mut right[0..(end1 - start1)], &mut left[start2..end2]))
        }
    }

    pub fn get_two(&self, entity1: Entity, entity2: Entity) -> Option<(&[u8], &[u8])> {
        assert!(entity1 != entity2);
        let idx1 = self.sparse.get(entity1 as usize)?;
        if *idx1 == usize::MAX {
            return None;
        }
        let idx2 = self.sparse.get(entity2 as usize)?;
        if *idx2 == usize::MAX {
            return None;
        }
        if self.layout.size == 0 {
            return Some((&[], &[]));
        }
        let start1 = idx1 * self.layout.size;
        let end1 = start1 + self.layout.size;
        let start2 = idx2 * self.layout.size;
        let end2 = start2 + self.layout.size;
        // dbg!(start1, end1);
        // dbg!(start2, end2);
        if start1 < start2 {
            assert!(end1 <= start2);
            let (left, right) = self.dense.split_at(start2 as usize);
            Some((&left[start1..end1], &right[0..(end2 - start2)]))
        } else {
            assert!(end2 <= start1);
            let (left, right) = self.dense.split_at(start1 as usize);
            Some((&right[0..(end1 - start1)], &left[start2..end2]))
        }
    }

    // fn get_two_mut(&mut self, a: Entity, b: Entity) -> Option<(&mut [u8], &mut [u8])> {
    //     get_two_mut(&mut self.dense, a, b)
    // }

    // fn get_two_mut<T>(&mut self, slice: &mut [T], i: Entity, j: Entity) -> (&mut T, &mut T) {
    //     assert!(i != j);

    //     if i < j {
    //         let (left, right) = slice.split_at_mut(j as usize);
    //         (&mut left[i as usize], &mut right[0])
    //     } else {
    //         let (left, right) = slice.split_at_mut(i as usize);
    //         (&mut right[0], &mut left[j as usize])
    //     }
    // }

    pub fn remove(&mut self, entity: Entity) -> bool {
        // TODO(anissen): DRY logic around entity existence check
        // TODO(anissen): It's probably faster to use bitmap.contains
        let idx = match self.sparse.get(entity as usize) {
            Some(i) if *i != usize::MAX => *i,
            _ => return false,
        };

        let last = self.entities.len() - 1;
        self.entities.swap(idx, last);

        let moved_entity = self.entities[idx];
        self.sparse[moved_entity as usize] = idx;
        self.sparse[entity as usize] = usize::MAX;

        if self.layout.size > 0 && idx < last {
            let s0 = idx * self.layout.size;
            let s1 = last * self.layout.size;

            let (left, right) = self.dense.split_at_mut(s1);
            let row0 = &mut left[s0..s0 + self.layout.size];
            let row1 = &mut right[0..self.layout.size];
            row0.swap_with_slice(row1);
        }

        self.bitset.unset(entity);
        self.entities.pop();
        true
    }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &[u8])> {
        self.entities.iter().enumerate().map(|(i, &e)| {
            let bytes = if self.layout.size == 0 {
                &[]
            } else {
                let start = i * self.layout.size;
                let end = start + self.layout.size;
                &self.dense[start..end]
            };
            (e, bytes)
        })
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter().copied()
    }
}
