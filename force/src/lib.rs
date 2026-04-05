use elements::{ComponentId, Entity, world::World};

#[derive(Clone, Copy, Debug)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Debug)]
struct Particle {
    position: Position,
    old_position: Position,
    mass: f32,
    friction: f32,
    pinned: bool,
}

impl Particle {
    fn new(position: Position) -> Self {
        Self {
            position,
            old_position: position,
            mass: 1.0,
            friction: 0.97,
            pinned: false,
        }
    }

    fn update(&mut self, dt: f32) {
        if self.pinned {
            return;
        }
        let velocity_x = (self.position.x - self.old_position.x) * dt * self.friction;
        let velocity_y = (self.position.y - self.old_position.y) * dt * self.friction;
        self.old_position = self.position;
        self.position.x += velocity_x;
        self.position.y += velocity_y;
    }
}

enum ConstrainType {
    FixedDistance,
    MinDistance,
    MaxDistance,
}

struct Constraint {
    type_: ConstrainType,
    start: usize, //&'a mut Particle,
    end: usize,   //&'a mut Particle,
    stiffness: f32,
    length: f32,
}

impl Constraint {
    fn update(&mut self, a: &mut Particle, b: &mut Particle) {
        if a.pinned && b.pinned {
            return;
        }

        // calculate the distance between two particles
        let dx = b.position.x - a.position.x;
        let dy = b.position.y - a.position.y;
        let dist = (dx * dx + dy * dy).sqrt();
        let diff = (self.length - dist) / dist * self.stiffness;

        // getting the offset of the points
        let offset_x = dx * diff * 0.5;
        let offset_y = dy * diff * 0.5;

        // calculate mass
        let total_mass = a.mass + b.mass;

        // and finally apply the offset with calculated mass
        if !a.pinned {
            let relative_mass_end = b.mass / total_mass;
            a.position.x -= offset_x * relative_mass_end;
            a.position.y -= offset_y * relative_mass_end;
        }
        if !b.pinned {
            let relative_mass_start = a.mass / total_mass;
            b.position.x += offset_x * relative_mass_start;
            b.position.y += offset_y * relative_mass_start;
        }
    }
}

struct Simulation {
    particles: Vec<Particle>,
    constraints: Vec<Constraint>,
    frames: usize,
    iterations_per_frame: usize,
}

impl Simulation {
    pub fn new(
        particles: Vec<Particle>,
        constraints: Vec<Constraint>,
        frames: usize,
        iterations_per_frame: usize,
    ) -> Self {
        Self {
            particles,
            constraints,
            frames,
            iterations_per_frame,
        }
    }

    pub fn run(&mut self) {
        for frame in 0..self.frames {
            println!("=== frame {} ===", frame);

            let dt = 0.1;

            for particle in &mut self.particles {
                particle.update(dt);
            }

            for _ in 0..self.iterations_per_frame {
                for constraint in &mut self.constraints {
                    let (a, b) = get_two_mut(&mut self.particles, constraint.start, constraint.end);
                    constraint.update(a, b);
                }
            }

            for particle in &self.particles {
                println!("{:?}", particle.position);
            }
        }
    }

    // add particle
    pub fn add_particle(&mut self, particle: Particle) {
        self.particles.push(particle);
    }

    // remove particle
    pub fn remove_particle(&mut self, index: usize) {
        self.particles.swap_remove(index);
    }

    // add constraint
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }
}

fn get_two_mut<T>(slice: &mut [T], i: usize, j: usize) -> (&mut T, &mut T) {
    assert!(i != j);

    if i < j {
        let (left, right) = slice.split_at_mut(j);
        (&mut left[i], &mut right[0])
    } else {
        let (left, right) = slice.split_at_mut(i);
        (&mut right[0], &mut left[j])
    }
}

pub fn update_positions(
    world: &mut World,
    position_id: ComponentId,
    last_position_id: ComponentId,
) {
    let mut results = world.query(&vec![position_id, last_position_id], &vec![]);

    // For each entity e
    //      update the position of e in World
    //
    // For each entity e1,
    //      for each other entity e2
    //          filter by broad phase, narrow phase
    //          resolve collision for e1 <=> e2 (update positions, create event?)
    //          update e2 in World
    //      update e1 in World

    // Update entity positions
    while let Some(entity) = results.next() {
        if let Ok([position_column, last_position_column]) = results
            .columns
            .get_disjoint_mut([position_id as usize, last_position_id as usize])
        {
            let position_data = position_column.get_mut(entity).unwrap();
            let last_position_data = last_position_column.get_mut(entity).unwrap();

            // "Constants"
            let dt = 0.1;
            let friction = 0.97;

            // TODO(anissen): I would like a more highlevel read/write API than operating on bytes
            let x = read_f32(&position_data[0..4]);
            let y = read_f32(&position_data[4..8]);
            let last_x = read_f32(&last_position_data[0..4]);
            let last_y = read_f32(&last_position_data[4..8]);
            let velocity_x = (x - last_x) * dt * friction;
            let velocity_y = (y - last_y) * dt * friction;

            last_position_data.copy_from_slice(&position_data);

            let new_position_x = x + velocity_x;
            let new_position_y = y + velocity_y;
            let new_position_data = [f32_bytes(new_position_x), f32_bytes(new_position_y)].concat();

            position_data.copy_from_slice(&new_position_data);

            // println!(
            //     "Entity {}: position {}, {}",
            //     entity, new_position_x, new_position_y
            // );
        } else {
            panic!("cannot get columns");
        }
    }
}

// Would be nice to have something like
// fn handle_collisions(query1, query2)
// e.g. (enemy, position) <=> (enemy, position)
// e.g. (enemy, position) <=> (player, position)
// e.g. (player, position) <=> (pickup, position)

pub fn handle_collisions(world: &mut World, position_id: ComponentId, physics_id: ComponentId) {
    // TODO: This should also include radius etc. (a physics component)
    let results = world.query(&vec![position_id, physics_id], &vec![]);
    let entities = results.collect::<Vec<_>>();

    // Handle collisions
    for (entity_index, entity) in entities.iter().enumerate() {
        for other_entity_index in entity_index + 1..entities.len() {
            let other_entity = entities[other_entity_index];
            let (radius, other_radius) = world
                .get_column(physics_id)
                .get_two(*entity, other_entity)
                .unwrap();
            let radius = read_f32(&radius[0..4]);
            let other_radius = read_f32(&other_radius[0..4]);

            let (pos, other_pos) = world
                .get_column_mut(position_id)
                .get_two_mut(*entity, other_entity)
                .unwrap();
            let mut pos_x = read_f32(&pos[0..4]);
            let mut pos_y = read_f32(&pos[4..8]);
            let mut other_pos_x = read_f32(&other_pos[0..4]);
            let mut other_pos_y = read_f32(&other_pos[4..8]);

            let minimum_distance = radius + other_radius; //200.0;
            let stiffness = 1.0;
            let max_iterations = 10;

            // TODO(anissen): Handle different kinds of constraints

            // iterations
            for _ in 0..max_iterations {
                // TODO(anissen): Should iterations be here or outside entire nested loop?
                // calculate the distance between two particles
                let dx = other_pos_x - pos_x;
                let dy = other_pos_y - pos_y;
                let dist = (dx * dx + dy * dy).sqrt();
                if dist > minimum_distance {
                    // println!("no collision");
                    break;
                }
                let dist_diff = minimum_distance - dist;
                // println!("collision: {}", dist_diff);
                let diff = dist_diff / dist * stiffness;

                // getting the offset of the points
                let offset_x = dx * diff * 0.5;
                let offset_y = dy * diff * 0.5;

                pos_x -= offset_x * 0.5;
                pos_y -= offset_y * 0.5;
                other_pos_x += offset_x * 0.5;
                other_pos_y += offset_y * 0.5;
            }

            pos.copy_from_slice(&[f32_bytes(pos_x), f32_bytes(pos_y)].concat());
            other_pos.copy_from_slice(&[f32_bytes(other_pos_x), f32_bytes(other_pos_y)].concat());

            // println!("Entity {} at ({}, {})", entity, pos_x, pos_y);
        }
    }
}

pub fn handle_links(world: &mut World, link_id: ComponentId, position_id: ComponentId) {
    let results = world.query(&vec![link_id], &vec![]);
    let entities = results.collect::<Vec<_>>();

    // while let Some(entity) = results.next() {
    for entity in entities {
        let link = world.get_column(link_id).get(entity).unwrap();
        let entity_a = read_u32(&link[0..4]);
        let entity_b = read_u32(&link[4..8]);
        let length = read_f32(&link[8..12]);

        let (pos_a, pos_b) = world
            .get_column_mut(position_id)
            .get_two_mut(entity_a, entity_b)
            .unwrap();
        let mut pos_a_x = read_f32(&pos_a[0..4]);
        let mut pos_a_y = read_f32(&pos_a[4..8]);
        let mut pos_b_x = read_f32(&pos_b[0..4]);
        let mut pos_b_y = read_f32(&pos_b[4..8]);

        let stiffness = 1.0;
        let max_iterations = 5;
        let acceptable_epsilon = 0.1;

        // TODO(anissen): Handle different kinds of constraints

        // iterations
        for _ in 0..max_iterations {
            // TODO(anissen): Should iterations be here or outside entire nested loop?
            // calculate the distance between two particles
            let dx = pos_b_x - pos_a_x;
            let dy = pos_b_y - pos_a_y;
            let dist = (dx * dx + dy * dy).sqrt();
            let dist_diff = length - dist;
            if dist_diff.abs() < acceptable_epsilon {
                break;
            }

            // println!("collision: {}", dist_diff);
            let diff = dist_diff / dist * stiffness;

            // getting the offset of the points
            let offset_x = dx * diff * 0.5;
            let offset_y = dy * diff * 0.5;

            pos_a_x -= offset_x * 0.5;
            pos_a_y -= offset_y * 0.5;
            pos_b_x += offset_x * 0.5;
            pos_b_y += offset_y * 0.5;
        }

        pos_a.copy_from_slice(&[f32_bytes(pos_a_x), f32_bytes(pos_a_y)].concat());
        pos_b.copy_from_slice(&[f32_bytes(pos_b_x), f32_bytes(pos_b_y)].concat());
    }
}

// pub fn simulation() {
//     let mut particles = Vec::new();
//     particles.push(Particle::new(Position { x: 0.0, y: 0.0 }));
//     particles.push(Particle::new(Position { x: 1.0, y: 1.0 }));

//     let mut constraints = Vec::new();
//     constraints.push(Constraint {
//         type_: ConstrainType::FixedDistance,
//         start: &mut particles[0],
//         end: &mut particles[1],
//         length: 1.0,
//         stiffness: 1.0,
//     });

//     let frames = 10;
//     let iterations_per_frame = 10;

//     for frame in 0..frames {
//         println!("=== frame {} ===", frame);

//         let dt = 0.1;

//         for particle in &mut particles {
//             particle.update(dt);
//         }

//         for _ in 0..iterations_per_frame {
//             for constraint in &mut constraints {
//                 constraint.update();
//             }
//         }

//         for particle in &particles {
//             println!("{:?}", particle.position);
//         }
//     }
// }

#[cfg(test)]
mod tests {
    use elements::{ComponentLayout, ComponentTypeId, EntityManager, FieldLayout};

    use super::*;

    // #[test]
    // fn it_works() {
    //     let mut particles = Vec::new();
    //     let p1 = Particle::new(Position { x: 0.0, y: 0.0 });
    //     let p2 = Particle::new(Position { x: 1.0, y: 1.0 });
    //     particles.push(p1);
    //     particles.push(p2);

    //     let mut constraints = Vec::new();
    //     constraints.push(Constraint {
    //         type_: ConstrainType::FixedDistance,
    //         start: 0,
    //         end: 1,
    //         length: 1.0,
    //         stiffness: 1.0,
    //     });

    //     let frames = 10;
    //     let iterations_per_frame = 10;
    //     let mut sim = Simulation::new(particles, constraints, frames, iterations_per_frame);
    //     sim.run();
    // }

    #[test]
    fn world_test() {
        let mut entity_manager = EntityManager::new();
        let mut world = World::new();
        let position_id: ComponentTypeId = 0;
        world.register_component(
            position_id,
            ComponentLayout::new(vec![
                FieldLayout {
                    name: "x".to_string(),
                    type_id: 2, // TODO(anissen): Fix this hack
                    size: 4,    // TODO(anissen): Fix this hack
                },
                FieldLayout {
                    name: "y".to_string(),
                    type_id: 2,
                    size: 4,
                },
            ]),
        );
        let last_position_id: ComponentTypeId = 1;
        world.register_component(
            last_position_id,
            ComponentLayout::new(vec![
                FieldLayout {
                    name: "x".to_string(),
                    type_id: 2,
                    size: 4,
                },
                FieldLayout {
                    name: "y".to_string(),
                    type_id: 2,
                    size: 4,
                },
            ]),
        );
        // Physics marker (no data)
        let physics_id: ComponentTypeId = 2;
        world.register_component(physics_id, ComponentLayout::new(vec![]));

        // Create a few entities
        let e0 = entity_manager.create();
        let e1 = entity_manager.create();
        let e2 = entity_manager.create();

        // Add components
        world.insert(position_id, e0, &position(1.0, 1.0));
        world.insert(last_position_id, e0, &last_position(0.0, 0.0));
        world.insert(physics_id, e0, &[]);

        world.insert(position_id, e1, &position(9.0, 9.0));
        world.insert(last_position_id, e1, &last_position(10.0, 10.0));
        world.insert(physics_id, e1, &[]);

        world.insert(position_id, e2, &position(3.0, 3.0));

        /*
        Ideas for API for force from VM:
        - Use World directly, querying the needed entities and decoding/encoding components
        - Operate on an entirely abstract set of data, handling particles and constraints and returning a changed set of particles
        - Use World but make a more highlevel API for easier interop
        */

        for frame in 0..3 {
            println!("--- Frame {} ---", frame);

            let mut results =
                world.query(&vec![position_id, last_position_id, physics_id], &vec![]);

            // For each entity e
            //      update the position of e in World
            //
            // For each entity e1,
            //      for each other entity e2
            //          filter by broad phase, narrow phase
            //          resolve collision for e1 <=> e2 (update positions, create event?)
            //          update e2 in World
            //      update e1 in World

            // Update entity positions
            while let Some(entity) = results.next() {
                if let Ok([position_column, last_position_column]) = results
                    .columns
                    .get_disjoint_mut([position_id as usize, last_position_id as usize])
                {
                    let position_data = position_column.get_mut(entity).unwrap();
                    let last_position_data = last_position_column.get_mut(entity).unwrap();

                    // "Constants"
                    let dt = 0.1;
                    let friction = 0.97;

                    // TODO(anissen): I would like a more highlevel read/write API than operating on bytes
                    let x = read_f32(&position_data[0..4]);
                    let y = read_f32(&position_data[4..8]);
                    let last_x = read_f32(&last_position_data[0..4]);
                    let last_y = read_f32(&last_position_data[4..8]);
                    let velocity_x = (x - last_x) * dt * friction;
                    let velocity_y = (y - last_y) * dt * friction;

                    last_position_data.copy_from_slice(&position_data);

                    let new_position_x = x + velocity_x;
                    let new_position_y = y + velocity_y;
                    let new_position_data =
                        [f32_bytes(new_position_x), f32_bytes(new_position_y)].concat();

                    position_data.copy_from_slice(&new_position_data);

                    // println!(
                    //     "Entity {}: position {}, {}",
                    //     entity, new_position_x, new_position_y
                    // );
                } else {
                    panic!("cannot get columns");
                }
            }

            // TODO: This should also include radius etc. (a physics component)
            let results = world.query(&vec![position_id, physics_id], &vec![]);
            let entities = results.collect::<Vec<_>>();

            // Handle collisions
            for (entity_index, entity) in entities.iter().enumerate() {
                for other_entity_index in entity_index + 1..entities.len() {
                    let other_entity = entities[other_entity_index];
                    let (pos, other_pos) = world
                        .get_column_mut(position_id)
                        .get_two_mut(*entity, other_entity)
                        .unwrap();
                    let mut pos_x = read_f32(&pos[0..4]);
                    let mut pos_y = read_f32(&pos[4..8]);
                    let mut other_pos_x = read_f32(&other_pos[0..4]);
                    let mut other_pos_y = read_f32(&other_pos[4..8]);

                    let minimum_distance = 15.0;
                    let stiffness = 1.0;

                    // TODO(anissen): Handle different kinds of constraints

                    // iterations
                    for _ in 0..10 {
                        // calculate the distance between two particles
                        let dx = other_pos_x - pos_x;
                        let dy = other_pos_y - pos_y;
                        let dist = (dx * dx + dy * dy).sqrt();
                        if dist > minimum_distance {
                            // println!("no collision");
                            break;
                        }
                        let dist_diff = minimum_distance - dist;
                        // println!("collision: {}", dist_diff);
                        let diff = dist_diff / dist * stiffness;

                        // getting the offset of the points
                        let offset_x = dx * diff * 0.5;
                        let offset_y = dy * diff * 0.5;

                        pos_x -= offset_x * 0.5;
                        pos_y -= offset_y * 0.5;
                        other_pos_x += offset_x * 0.5;
                        other_pos_y += offset_y * 0.5;
                    }

                    pos.copy_from_slice(&[f32_bytes(pos_x), f32_bytes(pos_y)].concat());
                    other_pos.copy_from_slice(
                        &[f32_bytes(other_pos_x), f32_bytes(other_pos_y)].concat(),
                    );

                    // println!("Entity {} at ({}, {})", entity, pos_x, pos_y);
                }
            }
        }
    }
}

fn f32_bytes(x: f32) -> [u8; 4] {
    x.to_be_bytes()
}
fn read_f32(b: &[u8]) -> f32 {
    f32::from_be_bytes(b.try_into().unwrap())
}
fn read_u32(b: &[u8]) -> u32 {
    u32::from_be_bytes(b.try_into().unwrap())
}

fn position(x: f32, y: f32) -> Vec<u8> {
    [f32_bytes(x), f32_bytes(y)].concat()
}

fn last_position(last_x: f32, last_y: f32) -> Vec<u8> {
    [f32_bytes(last_x), f32_bytes(last_y)].concat()
}
