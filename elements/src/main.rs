use elements::{
    ComponentLayout, ComponentTypeId, Entity, EntityManager, FieldLayout, world::World,
};

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

// ------------------------------------------------------------
// Packed component layout description
// ------------------------------------------------------------

// #[derive(Debug)]
// pub struct FieldLayout {
//     pub name: String,
//     pub offset: usize,
//     pub size: usize,
// }

// #[derive(Debug)]
// pub struct ComponentLayout {
//     pub size: usize,              // packed byte size
//     pub fields: Vec<FieldLayout>, // ordered field offsets
// }

// // ------------------------------------------------------------
// // Helpers: fixed-size types
// // ------------------------------------------------------------

// fn primitive_size(ty: &Type) -> usize {
//     match ty {
//         Type::Boolean => 1,
//         Type::Integer => 4,
//         Type::Float => 4,
//         Type::String => 8, // VM string handle
//         Type::Tag => 0,
//         Type::List => 16,    // pointer + len
//         Type::Function => 8, // function handle
//         Type::Component => unreachable!("Handled separately"),
//     }
// }

// // ------------------------------------------------------------
// // Packed layout generator (recursive, no padding)
// // ------------------------------------------------------------

// pub fn layout_component(
//     def: &ComponentDefinition,
//     components: &HashMap<String, ComponentDefinition>,
// ) -> ComponentLayout {
//     let mut fields = Vec::new();
//     let mut offset = 0usize;

//     for prop in &def.properties {
//         let size = match &prop.type_ {
//             Type::Component => {
//                 // nested component → compute recursively
//                 let nested_def = components
//                     .get(&prop.name)
//                     .unwrap_or_else(|| panic!("Unknown nested component {}", prop.name));

//                 let nested_layout = layout_component(nested_def, components);
//                 nested_layout.size
//             }
//             other => primitive_size(other),
//         };

//         fields.push(FieldLayout {
//             name: prop.name.clone(),
//             offset,
//             size,
//         });

//         offset += size; // packed: no padding
//     }

//     ComponentLayout {
//         size: offset,
//         fields,
//     }
// }

// TODO(anissen): Look at MemorySegment + MemoryLayout from JDK for API inspiration

fn main() {
    let mut entity_manager = EntityManager::new();
    let mut world = World::new();
    // Position { x: f32, y: f32 }
    let position_id: ComponentTypeId = 0;
    world.register_component(
        position_id,
        ComponentLayout::new(vec![
            FieldLayout {
                name: "x".to_string(),
                type_id: 0,
                size: 4,
            },
            FieldLayout {
                name: "y".to_string(),
                type_id: 0,
                size: 4,
            },
        ]),
    );
    // Velocity { dx: f32, dy: f32 }
    let velocity_id: ComponentTypeId = 1;
    world.register_component(
        velocity_id,
        ComponentLayout::new(vec![
            FieldLayout {
                name: "dx".to_string(),
                type_id: 0,
                size: 4,
            },
            FieldLayout {
                name: "dy".to_string(),
                type_id: 0,
                size: 4,
            },
        ]),
    );
    // Dead (no data)
    let dead_id: ComponentTypeId = 2;
    world.register_component(dead_id, ComponentLayout::new(vec![]));

    // Create a few entities
    let e0 = entity_manager.create();
    let e1 = entity_manager.create();
    let e2 = entity_manager.create();

    // Add components
    world.insert(position_id, e0, &position(0.01, 0.5));
    world.insert(velocity_id, e0, &velocity(3.3, 3.3));
    world.insert(velocity_id, e0, &velocity(1.0, 1.0));
    world.insert(dead_id, e0, &[]);

    world.insert(position_id, e1, &position(10.0, -5.0));
    world.insert(velocity_id, e1, &velocity(-2.0, 0.5));

    world.insert(position_id, e2, &position(3.0, 3.0));

    let e3 = entity_manager.create();
    world.insert(position_id, e3, &position(0.0, 0.0));
    world.insert(velocity_id, e3, &velocity(-1.0, -1.0));

    let e4 = entity_manager.create();
    world.insert(dead_id, e4, &[]);

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
        world.system(
            &vec![position_id, velocity_id],
            &vec![dead_id],
            movement_system,
        );

        world
            .iter(dead_id)
            .for_each(|(entity, _)| println!("Oh, no! Entity {} is dead!", entity));

        world.remove(dead_id, e0);

        world.insert(dead_id, e1, &[]);
        world.insert(velocity_id, e2, &velocity(1.0, 1.0));
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
