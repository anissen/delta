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

    /// Creates a new BitSet capable of holding `len` bits (all true).
    // pub fn all_set(len: usize) -> Self {
    //     let blocks = (len + 63) / 64;
    //     Self {
    //         data: vec![1; blocks],
    //         len,
    //     }
    // }

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
pub struct ComponentColumn {
    dense_components: Vec<Component>,
    dense_entities: Vec<Entity>,
    sparse: Vec<Option<usize>>,
    bitset: BitSet,
}

impl ComponentColumn {
    fn new() -> Self {
        Self {
            dense_components: Vec::new(),
            dense_entities: Vec::new(),
            sparse: Vec::new(),
            bitset: BitSet::new(3), // Used to be 64
        }
    }

    fn insert(&mut self, entity: Entity, component: Component) {
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
        // id < self.sparse.len() && self.sparse[id].is_some()
        id < self.bitset.len() && self.bitset.get(id)
    }

    fn get(&self, entity: Entity) -> Option<&Component> {
        let id = entity as usize;
        self.sparse
            .get(id)?
            .map(|index| &self.dense_components[index])
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut Component> {
        let id = entity as usize;
        self.sparse
            .get(id)?
            .map(|index| &mut self.dense_components[index])
    }

    fn remove(&mut self, entity: Entity) -> Option<Component> {
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

    fn iter(&self) -> impl Iterator<Item = (&Entity, &Component)> {
        self.dense_entities.iter().zip(&self.dense_components)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = (&Entity, &mut Component)> {
        self.dense_entities.iter().zip(&mut self.dense_components)
    }
}

pub struct ComponentStorage {
    // component_sets: HashMap<ComponentId, ComponentColumn>, // TODO: Make this a Vec or SparseSet instead of a HashMap
    component_sets: Vec<ComponentColumn>,
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

    fn get(&self, component_id: &ComponentId) -> Option<&ComponentColumn> {
        self.component_sets.get(*component_id as usize)
    }

    fn get_many(&self, component_ids: Vec<&ComponentId>) -> Option<Vec<&ComponentColumn>> {
        let len = self.component_sets.len();
        let has_all = component_ids.iter().all(|id| (**id as usize) < len);
        if !has_all {
            return None;
        }

        let mut index: u32 = 0;
        let many: Vec<_> = self
            .component_sets
            .iter()
            .filter(|_| {
                let include = component_ids.contains(&&index);
                index += 1;
                include
            })
            .collect();

        Some(many)
    }

    fn get_many_mut(
        &mut self,
        component_ids: Vec<&ComponentId>,
    ) -> Option<Vec<&mut ComponentColumn>> {
        let len = self.component_sets.len();
        let has_all = component_ids.iter().all(|id| (**id as usize) < len);
        if !has_all {
            return None;
        }

        // split the self.component_sets vec into N mutable values, one for each index that matches component_ids
        let mut index: u32 = 0;

        // let ids: Vec<_> = component_ids.iter().map(|id| **id as usize).collect();
        // let many = self
        //     .component_sets
        //     .get_disjoint_mut(ids)
        // let many = self.component_sets.get_disjoint_mut([ids[0], ids[1]]);
        // let many = self
        //     .component_sets
        //     .split_mut(|_| {
        //         let include = component_ids.contains(&&index);
        //         index += 1;
        //         include
        //     })
        //     .map(|mut columns| columns[0])
        //     .collect::<Vec<&mut ComponentColumn>>();
        let many: Vec<_> = self
            .component_sets
            .iter_mut()
            .filter(|_| {
                let include = component_ids.contains(&&index);
                index += 1;
                include
            })
            .collect();

        // [].mut
        Some(many)
    }

    fn entities(&self, component_ids: Vec<&ComponentId>) -> Vec<Entity> {
        if component_ids.is_empty() {
            return Vec::new();
        }

        if let Some(components) = self.get_many(component_ids) {
            let first_set = components.first().unwrap();

            if components.len() == 1 {
                return first_set.dense_entities.clone();
            }

            let mut intersection = first_set.bitset.clone();
            components
                .iter()
                .map(|col| &col.bitset)
                .for_each(|bitset| intersection.intersect_with(bitset));
            println!("Intersection: {:?}", intersection);
            intersection.print();
            println!("First Set: {:?}", first_set);

            first_set
                .dense_entities
                .iter()
                // .enumerate()
                .filter(|entity| intersection.get(**entity as usize))
                .map(|entity| *entity)
                .collect()
        } else {
            Vec::new()
        }
    }

    // fn get_two(
    //     &self,
    //     component_id1: &ComponentId,
    //     component_id2: &ComponentId,
    // ) -> Option<(&ComponentColumn, &ComponentColumn)> {
    //     let column1 = self.component_sets.get(component_id1)?;
    //     let column2 = self.component_sets.get(component_id2)?;
    //     Some((column1, column2))
    // }

    // fn get_two_mut(
    //     &mut self,
    //     component_id1: &ComponentId,
    //     component_id2: &ComponentId,
    // ) -> (&mut ComponentColumn, &mut ComponentColumn) {
    //     let [a, b] = self
    //         .component_sets
    //         .get_disjoint_mut([component_id1, component_id2]);
    //     (a.unwrap(), b.unwrap())
    // }

    // fn get_two_mut_iter(
    //     &mut self,
    //     component_id1: &ComponentId,
    //     component_id2: &ComponentId,
    // ) -> Vec<(&Component, &Component)> {
    //     let [first, second] = self
    //         .component_sets
    //         .get_disjoint_mut([component_id1, component_id2]);
    //     // let first = self.component_sets.get_mut(component_id1).unwrap();
    //     let first = &first.unwrap();
    //     let second = &second.unwrap();
    //     let entities = &first.dense_entities;
    //     // let second = self.component_sets.get_mut(component_id2).unwrap();
    //     let mut rows = Vec::new();
    //     let mut index = 0;
    //     for &entity in entities {
    //         let id = entity as usize;
    //         if first.sparse.get(id).is_some() && second.sparse.get(id).is_some() {
    //             rows.push((
    //                 &first.dense_components[index],
    //                 &second.dense_components[index],
    //             ));
    //         }
    //         index += 1;
    //     }
    //     rows
    // }

    // fn get_mut(&mut self, component_id: &ComponentId) -> Option<&mut ComponentColumn> {
    //     self.component_sets.get_mut(component_id)
    // }

    fn remove(&mut self, entity: Entity, component_id: ComponentId) -> Option<Component> {
        self.component_sets
            .get_mut(component_id as usize)?
            .remove(entity)
    }

    // fn iter(&self, component_id: ComponentId) -> impl Iterator<Item = (&Entity, &Component)> {
    //     self.component_sets.get(&component_id).unwrap().iter()
    // }

    // fn iter_mut(
    //     &mut self,
    //     component_id: ComponentId,
    // ) -> impl Iterator<Item = (&Entity, &mut Component)> {
    //     self.component_sets
    //         .get_mut(&component_id)
    //         .unwrap()
    //         .iter_mut()
    // }
}

#[derive(Debug)]
enum Value {
    Marker, // TODO(anissen): Should not be necessary to have for creating a marker component.
    Float(f32),
    Integer(i32),
    Boolean(bool),
}

impl Value {
    fn as_float(&self) -> f32 {
        match self {
            Value::Float(f) => *f,
            _ => panic!("Cannot convert non-float value to float"),
        }
    }
}

#[derive(Debug)]
struct Component {
    values: HashMap<String, Value>, // TODO(anissen): Could be mapped to Vec<Value> where field names are mapped to indexes.
}

type ComponentId = u32;

const POSITION_ID: ComponentId = 0;
const VELOCITY_ID: ComponentId = 1;
const DEAD_ID: ComponentId = 2;

fn movement_system(components: &mut ComponentStorage) {
    // Collect entities and velocity values that need updating
    let component_ids = vec![&POSITION_ID, &VELOCITY_ID];
    let entities = components.entities(vec![&POSITION_ID, &VELOCITY_ID]);
    if let Some(mut cols) = components.get_many_mut(component_ids) {
        let (first, rest) = cols.split_at_mut(1);
        let positions = &mut first[0];
        let pos_dense = &mut positions.dense_components;
        let velocities = &rest[0];

        // let iter = entities.iter().map(|entity| positions.)

        // let iter = &positions
        //     .iter_mut()
        //     .filter_map(|(entity, pos)| velocities.get(*entity).map(|vel| (pos, vel)));

        // let rows = (entity, pos, vel)

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

        // Apply updates
        // for (pos, vel) in cols {
        //     let dx = vel.values.get("dx").unwrap().as_float();
        //     let dy = vel.values.get("dy").unwrap().as_float();
        //     let pos_x = pos.values.get("x").unwrap().as_float();
        //     let pos_y = pos.values.get("y").unwrap().as_float();
        //     pos.values.insert("x".to_string(), Value::Float(pos_x + dx));
        //     pos.values.insert("y".to_string(), Value::Float(pos_y + dy));
        // }
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
        .insert(POSITION_ID as usize, ComponentColumn::new());
    components
        .component_sets
        .insert(VELOCITY_ID as usize, ComponentColumn::new());
    components
        .component_sets
        .insert(DEAD_ID as usize, ComponentColumn::new());

    // Create a few entities
    let e1 = entity_manager.create();
    let e2 = entity_manager.create();
    let e3 = entity_manager.create();

    // Add components
    components.set(e1, POSITION_ID, position(0.0, 0.0));
    components.set(e1, VELOCITY_ID, velocity(1.0, 1.0));
    components.set(e1, VELOCITY_ID, velocity(1.0, 1.0));
    components.set(e1, DEAD_ID, marker("is_dead"));

    dbg!(&components.get(&VELOCITY_ID));
    components.remove(e1, VELOCITY_ID);
    dbg!(&components.get(&VELOCITY_ID));

    components.set(e2, POSITION_ID, position(10.0, -5.0));
    components.set(e2, VELOCITY_ID, velocity(-2.0, 0.5));

    components.set(e3, POSITION_ID, position(3.0, 3.0));
    // e3 has no velocity—won’t move

    // Run the movement system a few times
    for frame in 0..3 {
        println!("--- Frame {} ---", frame);
        movement_system(&mut components);

        let matching_entities = components.entities(vec![&POSITION_ID, &VELOCITY_ID]);
        println!("Matching entities: {:?}", matching_entities);

        let dead_components = components.get(&DEAD_ID);
        for (entity, pos) in components.get(&POSITION_ID).unwrap().iter() {
            println!(
                "Entity {}: Position = ({:.1}, {:.1})",
                entity,
                pos.values.get("x").unwrap().as_float(),
                pos.values.get("y").unwrap().as_float()
            );
            if let Some(dead) = dead_components
                && dead.has(*entity)
            {
                println!("Entity {} is dead", entity);
            }
        }
        components.remove(e1, DEAD_ID);

        components.set(e2, DEAD_ID, marker("is_dead"));
        components.set(e3, VELOCITY_ID, velocity(1.0, 1.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
