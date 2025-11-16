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
