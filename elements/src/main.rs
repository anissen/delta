pub struct EntityManager {
    next_id: Entity,
}

impl EntityManager {
    fn new() -> Self {
        EntityManager { next_id: 0 }
    }

    fn create(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

#[derive(Debug, Clone)]
pub struct BitSet {
    data: Vec<u64>,
    len: usize, // number of bits that are *logically* part of the set
}

impl BitSet {
    /// Creates a new BitSet capable of holding `len` bits (all false).
    pub fn new(len: usize) -> Self {
        let blocks = (len + 63) / 64;
        Self {
            data: vec![0; blocks],
            len,
        }
    }

    /// Returns the number of bits in the bitset.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Checks if the bitset is empty (no bits).
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Gets the value of a bit.
    pub fn get(&self, index: usize) -> bool {
        assert!(index < self.len, "index out of bounds");
        let block = index / 64;
        let bit = index % 64;
        (self.data[block] >> bit) & 1 == 1
    }

    /// Sets the bit to the given value (resizes automatically if needed).
    pub fn set(&mut self, index: usize, value: bool) {
        if index >= self.len {
            self.resize(index + 1);
        }
        let block = index / 64;
        let bit = index % 64;
        if value {
            self.data[block] |= 1 << bit;
        } else {
            self.data[block] &= !(1 << bit);
        }
    }

    /// Clears all bits (sets all to false).
    pub fn clear(&mut self) {
        for block in &mut self.data {
            *block = 0;
        }
    }

    /// Returns the number of bits set to 1.
    pub fn count_ones(&self) -> usize {
        self.data.iter().map(|b| b.count_ones() as usize).sum()
    }

    /// Resizes the bitset to contain `new_len` bits.
    /// Newly added bits are set to false.
    pub fn resize(&mut self, new_len: usize) {
        let new_blocks = (new_len + 63) / 64;
        if new_blocks > self.data.len() {
            self.data.resize(new_blocks, 0);
        }
        self.len = new_len;
    }

    /// Performs intersection (AND) with another BitSet in place.
    /// Truncates to the smaller length.
    pub fn intersect_with(&mut self, other: &BitSet) {
        let min_blocks = self.data.len().min(other.data.len());
        for i in 0..min_blocks {
            self.data[i] &= other.data[i];
        }
        for i in min_blocks..self.data.len() {
            self.data[i] = 0;
        }
        self.len = self.len.min(other.len);
    }

    /// Performs relative complement (NOT) with another BitSet in place.
    /// Truncates to the smaller length.
    pub fn not_with(&mut self, other: &BitSet) {
        let min_blocks = self.data.len().min(other.data.len());
        for i in 0..min_blocks {
            // if other.data[i] == 1 {
            //     self.data[i] = 0;
            // }
            self.data[i] &= !other.data[i];
        }
        for i in min_blocks..self.data.len() {
            self.data[i] = 0;
        }
        self.len = self.len.min(other.len);
    }

    /// Returns a new BitSet that is the intersection of two bitsets.
    pub fn intersection(&self, other: &BitSet) -> BitSet {
        let min_len = self.len.min(other.len);
        let mut result = BitSet::new(min_len);
        for i in 0..result.data.len() {
            result.data[i] = self.data[i] & other.data[i];
        }
        result
    }

    pub fn print(&self) {
        for i in 0..self.len() {
            println!("BitSet {}: {}", i, self.get(i));
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ComponentLayout {
    pub size: usize,
    pub align: usize,
}

pub type ComponentTypeId = u32;
pub type Entity = u32;

#[derive(Debug)]
pub struct Column {
    id: ComponentId, // TODO(anissen): Needs to be encoded in World instead
    layout: ComponentLayout,
    dense: Vec<u8>,
    entities: Vec<Entity>,
    sparse: Vec<usize>,
    bitset: BitSet,
}

impl Column {
    pub fn new(
        component_id: ComponentId,
        layout: ComponentLayout,
        initial_capacity: usize,
    ) -> Self {
        Self {
            id: component_id,
            layout,
            dense: vec![0; initial_capacity * layout.size.max(1)], // allow zero-size components
            entities: Vec::with_capacity(initial_capacity),
            sparse: vec![usize::MAX; initial_capacity],
            bitset: BitSet::new(64), // 64 chosen arbitrarily
        }
    }

    fn ensure_entity_capacity(&mut self, entity: Entity) {
        if entity as usize >= self.sparse.len() {
            self.sparse.resize(entity as usize + 1, usize::MAX);
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

        // Replace component
        if idx != usize::MAX {
            if self.layout.size > 0 {
                let start = idx * self.layout.size;
                let end = start + self.layout.size;
                self.dense[start..end].copy_from_slice(value_bytes);
            }
            return;
        }

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
        self.bitset.set(entity as usize, true);
    }

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

    pub fn remove(&mut self, entity: Entity) -> bool {
        let idx = match self.sparse.get(entity as usize) {
            Some(i) if *i != usize::MAX => *i,
            _ => return false,
        };

        let last = self.entities.len() - 1;
        self.entities.swap(idx, last);

        let moved_entity = self.entities[idx];
        self.sparse[moved_entity as usize] = idx;
        self.sparse[entity as usize] = usize::MAX;

        if self.layout.size > 0 {
            let s0 = idx * self.layout.size;
            let s1 = last * self.layout.size;

            let (left, right) = self.dense.split_at_mut(s1);
            let row0 = &mut left[s0..s0 + self.layout.size];
            let row1 = &mut right[0..self.layout.size];
            row0.swap_with_slice(row1);
        }

        self.bitset.set(entity as usize, false);
        self.entities.pop();
        true
    }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &[u8])> {
        self.entities.iter().enumerate().map(move |(i, &e)| {
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

#[derive(Debug)]
pub struct World {
    components: Vec<Column>,
}

impl World {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn register_component(&mut self, id: ComponentTypeId, layout: ComponentLayout) {
        let idx = id as usize;
        if self.components.len() <= idx {
            self.components.resize_with(idx + 1, || {
                // TODO(anissen): This is probably crap
                Column::new(
                    id,
                    ComponentLayout {
                        size: 0,
                        align: 1, /* TODO(anissen): Could this be 0? */
                    }, // placeholder
                    0,
                )
            });
        }
        self.components[idx] = Column::new(id, layout, 16);
    }

    pub fn insert(&mut self, id: ComponentTypeId, entity: Entity, data: &[u8]) {
        self.components[id as usize].insert(entity, data);
    }

    pub fn remove(&mut self, id: ComponentTypeId, entity: Entity) {
        self.components[id as usize].remove(entity);
    }

    pub fn get(&self, id: ComponentTypeId, entity: Entity) -> Option<&[u8]> {
        self.components[id as usize].get(entity)
    }

    pub fn get_mut(&mut self, id: ComponentTypeId, entity: Entity) -> Option<&mut [u8]> {
        self.components[id as usize].get_mut(entity)
    }

    pub fn iter(&self, id: ComponentTypeId) -> impl Iterator<Item = (Entity, &[u8])> + '_ {
        self.components[id as usize].iter()
    }

    fn system(
        &mut self,
        include: &Vec<ComponentId>,
        exclude: &Vec<ComponentId>,
        mut system: impl FnMut(Entity, &mut Vec<&mut [u8]>),
    ) {
        if include.is_empty() {
            return;
        }

        let exclude_columns: Vec<_> = self
            .components
            .iter()
            .filter(|c| exclude.contains(&c.id))
            .collect();

        // TODO(anissen): Could split in (first, rest) for optimization
        let exclude_bitset = if let Some(first) = exclude_columns.first() {
            let mut bitset = first.bitset.clone();
            self.components
                .iter()
                .filter(|c| exclude.contains(&c.id))
                .map(|col| &col.bitset)
                .for_each(|other| bitset.intersect_with(other));
            // dbg!(&bitset);
            Some(bitset)
        } else {
            None
        };

        let mut include_columns: Vec<_> = self
            .components
            .iter_mut()
            .filter(|c| include.contains(&c.id))
            .collect();

        // TODO(anissen): Could split in (first, rest) for optimization
        if let Some(first) = include_columns.first() {
            let mut intersection = first.bitset.clone();
            include_columns
                .iter()
                .map(|col| &col.bitset)
                .for_each(|bitset| intersection.intersect_with(bitset));

            if let Some(exclude_bitset) = exclude_bitset {
                intersection.not_with(&exclude_bitset);
            }

            let entities = first.entities.clone();
            for i in 0..intersection.len() {
                if intersection.get(i) {
                    let entity = entities[i];
                    let mut row: Vec<_> = include_columns
                        .iter_mut()
                        .flat_map(|col| col.get_mut(entity))
                        .collect();
                    system(entity, &mut row);
                }
            }
        }
    }
}

type ComponentId = u32;

fn f32_bytes(x: f32) -> [u8; 4] {
    x.to_le_bytes()
}
fn read_f32(b: &[u8]) -> f32 {
    f32::from_le_bytes(b.try_into().unwrap())
}

fn position(x: f32, y: f32) -> Vec<u8> {
    [f32_bytes(x), f32_bytes(y)].concat()
}

fn velocity(dx: f32, dy: f32) -> Vec<u8> {
    [f32_bytes(dx), f32_bytes(dy)].concat()
}

fn main() {
    let mut entity_manager = EntityManager::new();
    let mut world = World::new();
    // Position { x: f32, y: f32 }
    let POSITION: ComponentTypeId = 0;
    world.register_component(POSITION, ComponentLayout { size: 8, align: 4 });
    // Velocity { dx: f32, dy: f32 }
    let VELOCITY: ComponentTypeId = 1;
    world.register_component(VELOCITY, ComponentLayout { size: 8, align: 4 });
    // Dead (no data)
    let DEAD: ComponentTypeId = 2;
    world.register_component(
        DEAD,
        ComponentLayout {
            size: 0,
            align: 1, /* TODO(anissen): Maybe 0? */
        },
    );

    // Create a few entities
    let e0 = entity_manager.create();
    let e1 = entity_manager.create();
    let e2 = entity_manager.create();

    // Add components
    world.insert(POSITION, e0, &position(0.01, 0.5));
    world.insert(VELOCITY, e0, &velocity(1.0, 1.0));
    world.insert(VELOCITY, e0, &velocity(1.0, 1.0));
    world.insert(DEAD, e0, &[]);

    world.insert(POSITION, e1, &position(10.0, -5.0));
    world.insert(VELOCITY, e1, &velocity(-2.0, 0.5));

    world.insert(POSITION, e2, &position(3.0, 3.0));

    let e3 = entity_manager.create();
    world.insert(POSITION, e3, &position(0.0, 0.0));
    world.insert(VELOCITY, e3, &velocity(-1.0, -1.0));

    /*
     * frame 1:
     *  e1: 8, -4.5
     *  e3: -1, -1
     *
     * frame 2:
     *  e0: 1, 1.5
     *  e2: 4, 4
     *  e3: -2, -2
     *
     * frame 3:
     *  e0: 2, 2.5
     *  e2: 5, 5
     *  e3: -3, -3
     */

    /*
    Benchmark for 1_000_000 iterations without println

    hyperfine './target/release/elements'
    Benchmark 1: ./target/release/elements
      Time (mean ± σ):     435.6 ms ±   4.0 ms    [User: 431.2 ms, System: 2.9 ms]
      Range (min … max):   431.7 ms … 442.3 ms    10 runs
    */
    for frame in 0..3 {
        println!("--- Frame {} ---", frame);

        // TODO(anissen): We probably need to get the list of entities/components out, and then iterate?!?
        world.system(&vec![POSITION, VELOCITY], &vec![DEAD], movement_system);

        world
            .iter(DEAD)
            .for_each(|(entity, _)| println!("Oh, no! Entity {} is dead!", entity));

        world.remove(DEAD, e0);

        world.insert(DEAD, e1, &[]);
        world.insert(VELOCITY, e2, &velocity(1.0, 1.0));
    }
}

fn movement_system(entity: Entity, components: &mut Vec<&mut [u8]>) {
    let (first, rest) = components.split_at_mut(1);
    let pos = &mut first[0];
    let vel = &mut rest[0];
    let pos_x = read_f32(&pos[0..4]);
    let pos_y = read_f32(&pos[4..8]);
    let vel_x = read_f32(&vel[0..4]);
    let vel_y = read_f32(&vel[4..8]);

    let new_pos_x = pos_x + vel_x;
    let new_pos_y = pos_y + vel_y;
    let new_pos = [f32_bytes(new_pos_x), f32_bytes(new_pos_y)].concat();

    pos.copy_from_slice(&new_pos);

    println!(
        "Entity {} at ({}, {}) with velocity ({}, {})",
        entity, new_pos_x, new_pos_y, vel_x, vel_y
    );
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {}
}
