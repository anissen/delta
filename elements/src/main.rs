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

    pub fn insert(&mut self, entity: Entity, component: T) {
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
    }

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

pub struct ComponentStorage<T> {
    component_sets: HashMap<ComponentId, SparseSet<T>>,
}

impl<T> ComponentStorage<T> {
    pub fn new() -> Self {
        Self {
            component_sets: HashMap::new(),
        }
    }

    pub fn set(&mut self, entity: Entity, component_id: ComponentId, component: T) {
        self.component_sets
            .entry(component_id)
            .or_insert_with(|| SparseSet::new())
            .insert(entity, component);
    }

    pub fn get(&self, component_id: &ComponentId) -> Option<&SparseSet<T>> {
        self.component_sets.get(component_id)
    }

    pub fn get_mut(&mut self, component_id: &ComponentId) -> Option<&mut SparseSet<T>> {
        self.component_sets.get_mut(component_id)
    }

    pub fn remove(&mut self, entity: Entity, component_id: ComponentId) -> Option<T> {
        self.component_sets
            .get_mut(&component_id)
            .take()?
            .remove(entity)
    }

    pub fn iter(&self, component_id: ComponentId) -> impl Iterator<Item = (Entity, &T)> {
        self.component_sets.get(&component_id).unwrap().iter()
    }

    pub fn iter_mut(
        &mut self,
        component_id: ComponentId,
    ) -> impl Iterator<Item = (Entity, &mut T)> {
        self.component_sets
            .get_mut(&component_id)
            .unwrap()
            .iter_mut()
    }
}

#[derive(Debug, Clone, Copy)]
struct Component {
    // values: HashMap<String, f32>,
    x: f32,
    y: f32,
}

type ComponentId = u32;

pub struct World {
    components: ComponentStorage<Component>,
}

impl World {
    fn new() -> Self {
        Self {
            components: ComponentStorage::new(),
        }
    }

    fn set(&mut self, entity: Entity, component_id: ComponentId, component: Component) {
        self.components.set(entity, component_id, component)
    }
}

const POSITION_ID: ComponentId = 0;
const VELOCITY_ID: ComponentId = 1;

fn movement_system(world: &mut World) {
    let positions = world.components.get(&POSITION_ID).unwrap();
    let velocities = world.components.get(&VELOCITY_ID).unwrap();

    // for (entity, position) in positions.iter_mut() {
    //     if let Some(vel) = velocities.get(entity) {
    //         position.x += vel.x;
    //         position.y += vel.y;
    //     }
    // }

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
//     let positions = &mut world.components.get_mut(&POSITION_ID).unwrap();
//     let velocities = &world.components.get(&VELOCITY_ID).unwrap();

//     // let (positions, velocities) = (
//     //     &mut world.components.get_mut(&POSITION_ID).unwrap(),
//     //     &world.components.get(&VELOCITY_ID).unwrap(),
//     // );

//     // Choose the smaller set to iterate over
//     let iter = if positions.set.dense_entities.len() <= velocities.set.dense_entities.len() {
//         positions
//             .iter_mut()
//             .filter_map(|(entity, pos)| velocities.get(entity).map(|vel| (pos, vel)))
//             .collect::<Vec<_>>()
//     } else {
//         velocities
//             .iter()
//             .filter_map(|(entity, vel)| positions.get_mut(entity).map(|pos| (pos, vel)))
//             .collect::<Vec<_>>()
//     };

//     for (pos, vel) in iter {
//         pos.x += vel.x;
//         pos.y += vel.y;
//     }
// }

fn main() {
    let mut entity_manager = EntityManager::new();
    let mut world = World::new();

    // Create a few entities
    let e1 = entity_manager.create();
    let e2 = entity_manager.create();
    let e3 = entity_manager.create();

    // Add components
    world.set(e1, POSITION_ID, Component { x: 0.0, y: 0.0 });
    world.set(e1, VELOCITY_ID, Component { x: 1.0, y: 1.0 });
    world.set(e1, VELOCITY_ID, Component { x: 1.0, y: 1.0 });

    world.set(e2, POSITION_ID, Component { x: 10.0, y: -5.0 });
    world.set(e2, VELOCITY_ID, Component { x: -2.0, y: 0.5 });

    world.set(e3, POSITION_ID, Component { x: 3.0, y: 3.0 });
    // e3 has no velocity—won’t move

    // Run the movement system a few times
    for frame in 0..3 {
        println!("--- Frame {} ---", frame);
        movement_system(&mut world);

        for (entity, pos) in world.components.get(&POSITION_ID).unwrap().iter() {
            println!("Entity {}: Position = ({:.1}, {:.1})", entity, pos.x, pos.y);
        }
        world.set(e3, VELOCITY_ID, Component { x: 1.0, y: 1.0 });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
