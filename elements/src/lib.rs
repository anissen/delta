mod bitset;
mod column;
pub mod world;

pub type Entity = u32;
pub type ComponentId = u32;
pub type ComponentTypeId = u32;

#[derive(Clone, Copy, Debug)]
pub struct ComponentLayout {
    pub size: usize,
    pub align: usize,
}

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
