use std::collections::HashMap;

type Entity = u32;

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

#[derive(Debug)]
pub struct ComponentColumn<T> {
    id: ComponentId,
    dense_components: Vec<T>,
    dense_entities: Vec<Entity>,
    sparse: Vec<Option<usize>>,
    bitset: BitSet,
}

impl<T> ComponentColumn<T> {
    fn new(id: ComponentId) -> Self {
        Self {
            id,
            dense_components: Vec::new(),
            dense_entities: Vec::new(),
            sparse: Vec::new(),
            bitset: BitSet::new(64), // 64 chosen arbitrarily
        }
    }

    fn insert(&mut self, entity: Entity, component: T) {
        let id = entity as usize;

        if let Some(Some(index)) = self.sparse.get(id) {
            // Replace component
            self.dense_components[*index] = component;
        } else {
            // Insert component
            if id >= self.sparse.len() {
                self.sparse.resize(id + 1, None);
            }

            let index = self.dense_components.len();
            self.dense_components.push(component);
            self.dense_entities.push(entity);
            self.sparse[id] = Some(index);
        }

        self.bitset.set(id, true);
    }

    fn has(&self, entity: Entity) -> bool {
        let id = entity as usize;
        id < self.bitset.len() && self.bitset.get(id)
    }

    fn get(&self, entity: Entity) -> Option<&T> {
        let id = entity as usize;
        self.sparse
            .get(id)?
            .map(|index| &self.dense_components[index])
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let id = entity as usize;
        self.sparse
            .get(id)?
            .map(|index| &mut self.dense_components[index])
    }

    fn remove(&mut self, entity: Entity) -> Option<T> {
        let id = entity as usize;
        let index = self.sparse.get_mut(id)?.take()?;

        let last_index = self.dense_components.len() - 1;

        if index != last_index {
            self.dense_components.swap(index, last_index);
            self.dense_entities.swap(index, last_index);
            let moved_entity = self.dense_entities[index];
            self.sparse[moved_entity as usize] = Some(index);
        }

        self.bitset.set(id, false);
        self.dense_entities.pop();
        self.dense_components.pop()
    }

    fn iter(&self) -> impl Iterator<Item = (&Entity, &T)> {
        self.dense_entities.iter().zip(&self.dense_components)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (&Entity, &mut T)> {
        self.dense_entities.iter().zip(&mut self.dense_components)
    }
}

pub struct ComponentStorage {
    component_sets: Vec<ComponentColumn<Component>>,
}

impl ComponentStorage {
    fn new() -> Self {
        Self {
            component_sets: Vec::new(),
        }
    }

    fn set(&mut self, entity: Entity, component_id: ComponentId, component: Component) {
        self.component_sets[component_id as usize].insert(entity, component);
    }

    fn get(&self, component_id: &ComponentId) -> &ComponentColumn<Component> {
        self.component_sets.get(*component_id as usize).unwrap()
    }

    fn has(&self, component_id: &ComponentId) -> bool {
        (*component_id as usize) < self.component_sets.len()
    }

    fn get_many(&self, component_ids: &Vec<ComponentId>) -> Vec<&ComponentColumn<Component>> {
        self.component_sets
            .iter()
            .filter(|c| component_ids.contains(&c.id))
            .collect()
    }

    fn get_many_mut(
        &mut self,
        component_ids: &Vec<ComponentId>,
    ) -> Vec<&mut ComponentColumn<Component>> {
        self.component_sets
            .iter_mut()
            .filter(|c| component_ids.contains(&c.id))
            .collect()
    }

    // type EntityRow = (Entity, Vec<&mut Component>);
    // Vec<(Entity, Row)>
    // fn query(&mut self, include: &Vec<ComponentId>) -> Vec<(Entity, Vec<&mut Component>)> {
    //     if include.is_empty() {
    //         return Vec::new();
    //     }

    //     let mut matching_components: Vec<_> = self
    //         .component_sets
    //         .iter_mut()
    //         .filter(|c| include.contains(&c.id))
    //         .collect();

    //     let matching_entities = if let Some((first, rest)) = matching_components.split_first() {
    //         if rest.is_empty() {
    //             first.dense_entities.clone()
    //         } else {
    //             let mut intersection = first.bitset.clone();
    //             rest.iter()
    //                 .map(|col| &col.bitset)
    //                 .for_each(|bitset| intersection.intersect_with(bitset));

    //             // println!("Intersection: {:?}", intersection);
    //             // intersection.print();
    //             // println!("First Set: {:?}", first_set);

    //             first
    //                 .dense_entities
    //                 .iter()
    //                 .filter(|entity| intersection.get(**entity as usize))
    //                 .map(|entity| *entity)
    //                 .collect()
    //         }
    //     } else {
    //         Vec::new()
    //     };

    //     matching_entities
    //         .iter()
    //         .map(|entity| {
    //             let row: Vec<_> = matching_components
    //                 .iter_mut()
    //                 .filter_map(|c| c.get_mut(*entity))
    //                 .collect();
    //             (entity, row)
    //         })
    //         .collect()
    // }

    // type Row = Vec<&mut Component>;

    fn system(
        &mut self,
        include: &Vec<ComponentId>,
        exclude: &Vec<ComponentId>,
        mut system: impl FnMut(Entity, Vec<&mut Component>),
    ) {
        if include.is_empty() {
            return;
        }

        let exclude_columns: Vec<_> = self
            .component_sets
            .iter()
            .filter(|c| exclude.contains(&c.id))
            .collect();

        // TODO(anissen): Could split in (first, rest) for optimization
        let exclude_bitset = if let Some(first) = exclude_columns.first() {
            let mut bitset = first.bitset.clone();
            self.component_sets
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
            .component_sets
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

            let entities = first.dense_entities.clone();
            for i in 0..intersection.len() {
                if intersection.get(i) {
                    let entity = entities[i];
                    let row: Vec<_> = include_columns
                        .iter_mut()
                        .flat_map(|col| col.get_mut(entity))
                        .collect();
                    system(entity, row);
                }
            }
        }
    }

    fn entities(&self, component_ids: &Vec<ComponentId>) -> Vec<Entity> {
        if component_ids.is_empty() {
            return Vec::new();
        }

        let matching_components: Vec<_> = self
            .component_sets
            .iter()
            .filter(|c| component_ids.contains(&c.id))
            .collect();

        if let Some((first, rest)) = matching_components.split_first() {
            if rest.is_empty() {
                return first.dense_entities.clone();
            }

            let mut intersection = first.bitset.clone();
            rest.iter()
                .map(|col| &col.bitset)
                .for_each(|bitset| intersection.intersect_with(bitset));

            // println!("Intersection: {:?}", intersection);
            // intersection.print();
            // println!("First Set: {:?}", first_set);

            first
                .dense_entities
                .iter()
                .filter(|entity| intersection.get(**entity as usize))
                .map(|entity| *entity)
                .collect()
        } else {
            Vec::new()
        }
    }

    fn remove(&mut self, entity: Entity, component_id: ComponentId) -> Option<Component> {
        self.component_sets
            .get_mut(component_id as usize)?
            .remove(entity)
    }
}

#[derive(Debug)]
enum Value {
    Marker, // TODO(anissen): Should not be necessary to have for creating a marker component.
    Float(f32),
    // Integer(i32),
    // Boolean(bool),
}

impl Value {
    fn as_float(&self) -> f32 {
        match self {
            Value::Float(f) => *f,
            _ => panic!("Cannot convert non-float value to float"),
        }
    }
}

// #[derive(Debug)]
// union ValueX {
//     float: f32,
//     int: i32,
// }

#[derive(Debug)]
struct Component {
    values: HashMap<String, Value>, // TODO(anissen): Could be mapped to Vec<Value> where field names are mapped to indexes.
}

type ComponentId = u32;

const POSITION_ID: ComponentId = 0;
const VELOCITY_ID: ComponentId = 1;
const DEAD_ID: ComponentId = 2;

fn movement_system(components: &mut ComponentStorage) {
    let component_ids = vec![POSITION_ID, VELOCITY_ID];
    let entities = components.entities(&vec![POSITION_ID, VELOCITY_ID]);
    let mut cols = components.get_many_mut(&component_ids);
    let (first, rest) = cols.split_at_mut(1);
    let positions = &mut first[0];
    let pos_dense = &mut positions.dense_components;
    let velocities = &rest[0];

    for entity in entities {
        let id = entity as usize;
        let pos = &mut pos_dense[positions.sparse[id].unwrap()];
        let vel = &velocities.dense_components[velocities.sparse[id].unwrap()];

        let dx = vel.values.get("dx").unwrap().as_float();
        let dy = vel.values.get("dy").unwrap().as_float();
        let pos_x = pos.values.get("x").unwrap().as_float();
        let pos_y = pos.values.get("y").unwrap().as_float();
        pos.values.insert("x".to_string(), Value::Float(pos_x + dx));
        pos.values.insert("y".to_string(), Value::Float(pos_y + dy));
    }
}

fn position(x: f32, y: f32) -> Component {
    Component {
        values: HashMap::from([
            ("x".to_string(), Value::Float(x)),
            ("y".to_string(), Value::Float(y)),
        ]),
    }
}

fn velocity(dx: f32, dy: f32) -> Component {
    Component {
        values: HashMap::from([
            ("dx".to_string(), Value::Float(dx)),
            ("dy".to_string(), Value::Float(dy)),
        ]),
    }
}

fn marker(str: &str) -> Component {
    Component {
        values: HashMap::from([(str.to_string(), Value::Marker)]),
    }
}

fn main() {
    let mut entity_manager = EntityManager::new();
    let mut components = ComponentStorage::new();
    components
        .component_sets
        .insert(POSITION_ID as usize, ComponentColumn::new(POSITION_ID));
    components
        .component_sets
        .insert(VELOCITY_ID as usize, ComponentColumn::new(VELOCITY_ID));
    components
        .component_sets
        .insert(DEAD_ID as usize, ComponentColumn::new(DEAD_ID));

    // Create a few entities
    let e0 = entity_manager.create();
    let e1 = entity_manager.create();
    let e2 = entity_manager.create();

    // Add components
    components.set(e0, POSITION_ID, position(0.0, 0.0));
    components.set(e0, VELOCITY_ID, velocity(1.0, 1.0));
    components.set(e0, VELOCITY_ID, velocity(1.0, 1.0));
    components.set(e0, DEAD_ID, marker("is_dead"));

    // dbg!(&components.get(&VELOCITY_ID));
    components.remove(e0, VELOCITY_ID);
    // dbg!(&components.get(&VELOCITY_ID));

    components.set(e1, POSITION_ID, position(10.0, -5.0));
    components.set(e1, VELOCITY_ID, velocity(-2.0, 0.5));

    components.set(e2, POSITION_ID, position(3.0, 3.0));

    let e3 = entity_manager.create();
    components.set(e3, POSITION_ID, position(0.0, 0.0));
    components.set(e3, VELOCITY_ID, velocity(-1.0, -1.0));

    // Run the movement system a few times
    for frame in 0..3 {
        println!("--- Frame {} ---", frame);
        movement_system(&mut components);

        let matching_entities = components.entities(&vec![POSITION_ID, VELOCITY_ID]);
        println!("Entities with (pos, vel): {:?}", matching_entities);

        components.system(&vec![POSITION_ID], &vec![], |entity, components| {
            let pos = components.first().unwrap();
            println!(
                "Entity {}: Position = ({:.1}, {:.1})",
                entity,
                pos.values.get("x").unwrap().as_float(),
                pos.values.get("y").unwrap().as_float()
            );
        });

        components
            .get(&DEAD_ID)
            .iter()
            .for_each(|(entity, _)| println!("Oh, no! Entity {} is dead!", entity));

        components.remove(e0, DEAD_ID);

        components.set(e1, DEAD_ID, marker("is_dead"));
        components.set(e2, VELOCITY_ID, velocity(1.0, 1.0));

        println!();
    }

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
        components.system(
            &vec![POSITION_ID, VELOCITY_ID],
            &vec![DEAD_ID],
            |entity, mut components| {
                let (first, rest) = components.split_at_mut(1);
                let pos = &mut first[0];
                let vel = &mut rest[0];
                let pos_x = pos.values.get("x").unwrap().as_float();
                let pos_y = pos.values.get("y").unwrap().as_float();
                let vel_x = vel.values.get("dx").unwrap().as_float();
                let vel_y = vel.values.get("dy").unwrap().as_float();
                pos.values
                    .insert("x".to_string(), Value::Float(pos_x + vel_x));
                pos.values
                    .insert("y".to_string(), Value::Float(pos_y + vel_y));
                println!(
                    "Entity {}: Position = ({:.1}, {:.1})",
                    entity,
                    pos.values.get("x").unwrap().as_float(),
                    pos.values.get("y").unwrap().as_float()
                );
            },
        );
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn it_works() {}
}
