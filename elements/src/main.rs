use std::collections::HashMap;

type Entity = u32;

pub struct EntityManager {
    next_id: Entity,
}

impl EntityManager {
    pub fn new() -> Self {
        EntityManager { next_id: 0 }
    }

    pub fn create(&mut self) -> Entity {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

#[derive(Debug, Clone, Copy)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, Copy)]
struct Velocity {
    dx: f32,
    dy: f32,
}

pub struct ComponentStorage<T> {
    set: SparseSet<T>,
}

impl<T> ComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            set: SparseSet::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity, component: T) {
        self.set.insert(entity, component);
    }

    pub fn get(&self, entity: Entity) -> Option<&T> {
        self.set.get(entity)
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        self.set.get_mut(entity)
    }

    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        self.set.remove(entity)
    }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.set.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        self.set.iter_mut()
    }
}

#[derive(Debug, Clone, Copy)]
struct Component {
    // values: HashMap<String, f32>,
    x: f32,
    y: f32,
}

// impl Component {
//     fn new() -> Self {
//         Self {
//             values: HashMap::new(),
//         }
//     }

//     fn new(x: f32, y: f32) -> Self {
//         let mut values = HashMap::new();
//         values.insert("x".to_string(), x);
//         values.insert("y".to_string(), y);
//         Self { values }
//     }
// }

pub struct World {
    components: HashMap<u32, ComponentStorage<Component>>,
}

impl World {
    fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    fn add_position(&mut self, entity: Entity, component_id: u32, position: Position) {
        self.components
            .entry(component_id)
            .or_insert_with(|| ComponentStorage::new())
            .insert(
                entity,
                Component {
                    x: position.x,
                    y: position.y,
                },
            );
        // self.positions.insert(entity, position);
    }

    fn add_velocity(&mut self, entity: Entity, component_id: u32, velocity: Velocity) {
        self.components
            .entry(component_id)
            .or_insert_with(|| ComponentStorage::new())
            .insert(
                entity,
                Component {
                    x: velocity.dx,
                    y: velocity.dy,
                },
            );
    }
}

const POSITION_ID: u32 = 0;
const VELOCITY_ID: u32 = 1;

fn movement_system(world: &mut World) {
    let positions = world.components.get(&POSITION_ID).unwrap();
    let velocities = world.components.get(&VELOCITY_ID).unwrap();

    // Collect entities and velocity values that need updating
    let updates: Vec<(Entity, Component)> = positions
        .iter()
        .filter_map(|(entity, _pos)| velocities.get(entity).map(|vel| (entity, *vel)))
        .collect();

    // Apply updates
    let positions = world.components.get_mut(&POSITION_ID).unwrap();
    for (entity, vel) in updates {
        if let Some(pos) = positions.get_mut(entity) {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }
}

// fn movement_system(world: &mut World) {
//     let (positions, velocities) = (&mut world.positions, &world.velocities);

//     // Choose the smaller set to iterate over
//     let iter = if positions.set.dense_entities.len() <= velocities.set.dense_entities.len() {
//         positions
//             .iter_mut()
//             .filter_map(|(entity, pos)| velocities.get(entity).map(|vel| (pos, vel)))
//     } else {
//         velocities
//             .iter()
//             .filter_map(|(entity, vel)| positions.get_mut(entity).map(|pos| (pos, vel)))
//     };

//     for (pos, vel) in iter {
//         pos.x += vel.dx;
//         pos.y += vel.dy;
//     }
// }

pub struct SparseSet<T> {
    dense_components: Vec<T>,
    dense_entities: Vec<Entity>,
    sparse: Vec<Option<usize>>,
}

impl<T> SparseSet<T> {
    pub fn new() -> Self {
        Self {
            dense_components: Vec::new(),
            dense_entities: Vec::new(),
            sparse: Vec::new(),
        }
    }
}

impl<T> SparseSet<T> {
    pub fn insert(&mut self, entity: Entity, component: T) {
        let index = self.dense_components.len();
        let id = entity as usize;

        if id >= self.sparse.len() {
            self.sparse.resize(id + 1, None);
        }

        if self.sparse[id].is_some() {
            panic!("Entity {:?} already has component", entity);
        }

        self.dense_components.push(component);
        self.dense_entities.push(entity);
        self.sparse[id] = Some(index);
    }
}

impl<T> SparseSet<T> {
    pub fn get(&self, entity: Entity) -> Option<&T> {
        let id = entity as usize;
        self.sparse
            .get(id)?
            .map(|index| &self.dense_components[index])
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut T> {
        let id = entity as usize;
        self.sparse
            .get(id)?
            .map(|index| &mut self.dense_components[index])
    }
}

impl<T> SparseSet<T> {
    pub fn remove(&mut self, entity: Entity) -> Option<T> {
        let id = entity as usize;
        let index = self.sparse.get_mut(id)?.take()?;

        let last_index = self.dense_components.len() - 1;
        self.dense_components.swap(index, last_index);
        self.dense_entities.swap(index, last_index);

        let moved_entity = self.dense_entities[index];
        self.sparse[moved_entity as usize] = Some(index);

        self.dense_entities.pop();
        let removed = self.dense_components.pop();

        removed
    }
}

impl<T> SparseSet<T> {
    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.dense_entities
            .iter()
            .cloned()
            .zip(self.dense_components.iter())
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        self.dense_entities
            .iter()
            .cloned()
            .zip(self.dense_components.iter_mut())
    }
}

fn main() {
    let mut entity_manager = EntityManager::new();
    let mut world = World::new();

    // Create a few entities
    let e1 = entity_manager.create();
    let e2 = entity_manager.create();
    let e3 = entity_manager.create();

    // Add components
    world.add_position(e1, POSITION_ID, Position { x: 0.0, y: 0.0 });
    world.add_velocity(e1, VELOCITY_ID, Velocity { dx: 1.0, dy: 1.0 });

    world.add_position(e2, POSITION_ID, Position { x: 10.0, y: -5.0 });
    world.add_velocity(e2, VELOCITY_ID, Velocity { dx: -2.0, dy: 0.5 });

    world.add_position(e3, POSITION_ID, Position { x: 3.0, y: 3.0 });
    // e3 has no velocity—won’t move

    // Run the movement system a few times
    for frame in 0..3 {
        println!("--- Frame {} ---", frame);
        movement_system(&mut world);

        for (entity, pos) in world.components.get(&POSITION_ID).unwrap().iter() {
            println!("Entity {}: Position = ({:.1}, {:.1})", entity, pos.x, pos.y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
