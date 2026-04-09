use crate::{ComponentId, ComponentLayout, Entity, bitset::BitSet, column::Column};

// pub struct QueryResultMutIter<'a> {
//     iter: std::vec::IntoIter<(u32, Vec<&'a mut [u8]>)>,
// }

// impl<'a> QueryResultMutIter<'a> {
//     pub fn new(results: Vec<(Entity, Vec<&'a mut [u8]>)>) -> Self {
//         Self {
//             iter: results.into_iter(),
//         }
//     }

//     pub fn empty() -> Self {
//         Self {
//             iter: Vec::new().into_iter(),
//         }
//     }
// }

// impl<'a> Iterator for QueryResultMutIter<'a> {
//     type Item = (Entity, Vec<&'a mut [u8]>);

//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next()
//     }
// }

pub struct QueryResult {
    entities: std::vec::IntoIter<Entity>,
    pub component_ids: Vec<ComponentId>,
    pub is_empty: bool,
    // pub columns: Vec<&'a mut Column>,
}

impl QueryResult {
    pub fn new(entities: Vec<Entity>, component_ids: Vec<ComponentId>) -> Self {
        Self {
            is_empty: entities.is_empty(),
            entities: entities.into_iter(),
            component_ids,
        }
    }
}

impl Iterator for QueryResult {
    type Item = Entity;

    fn next(&mut self) -> Option<Self::Item> {
        self.entities.next()
    }
}

// --------------------

#[derive(Debug)]
pub struct World {
    components: Vec<Column>,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
        }
    }

    pub fn register_component(&mut self, id: ComponentId, layout: ComponentLayout) {
        let column = Column::new(id, layout, 16);
        let idx = id as usize;
        if idx < self.components.len() {
            self.components[idx] = column;
        } else {
            self.components.push(column);
        }
    }

    pub fn get_component_layout(&self, id: ComponentId) -> Option<&ComponentLayout> {
        self.components
            .get(id as usize)
            .map(|column| &column.layout)
    }

    pub fn insert(&mut self, id: ComponentId, entity: Entity, data: &[u8]) {
        self.components[id as usize].insert(entity, data);
    }

    pub fn remove(&mut self, id: ComponentId, entity: Entity) {
        self.components[id as usize].remove(entity);
    }

    pub fn destroy(&mut self, entity: Entity) {
        self.components.iter_mut().for_each(|column| {
            column.remove(entity);
        });
    }

    pub fn get_column(&self, id: ComponentId) -> &Column {
        &self.components[id as usize]
    }

    pub fn get_column_mut(&mut self, id: ComponentId) -> &mut Column {
        &mut self.components[id as usize]
    }

    pub fn get_two_columns_mut(
        &mut self,
        id1: ComponentId,
        id2: ComponentId,
    ) -> Result<[&mut Column; 2], std::slice::GetDisjointMutError> {
        self.components
            .get_disjoint_mut([id1 as usize, id2 as usize])
    }

    pub fn get(&self, id: ComponentId, entity: Entity) -> Option<&[u8]> {
        self.components[id as usize].get(entity)
    }

    pub fn get_mut(&mut self, id: ComponentId, entity: Entity) -> Option<&mut [u8]> {
        self.components[id as usize].get_mut(entity)
    }

    pub fn iter(&self, id: ComponentId) -> impl Iterator<Item = (Entity, &[u8])> + '_ {
        self.components[id as usize].iter()
    }

    // pub fn system(
    //     &mut self,
    //     include: &Vec<ComponentId>,
    //     exclude: &Vec<ComponentId>,
    //     mut system: impl FnMut(Entity, &mut Vec<&mut [u8]>),
    // ) {
    //     if include.is_empty() {
    //         return;
    //     }

    //     let exclude_columns: Vec<_> = self
    //         .components
    //         .iter()
    //         .filter(|c| exclude.contains(&c.id))
    //         .collect();

    //     // TODO(anissen): Could split in (first, rest) for optimization
    //     let exclude_bitset = if let Some(first) = exclude_columns.first() {
    //         let mut bitset = first.bitset.clone();
    //         self.components
    //             .iter()
    //             .filter(|c| exclude.contains(&c.id))
    //             .map(|col| &col.bitset)
    //             .for_each(|other| bitset.intersect_with(other));
    //         Some(bitset)
    //     } else {
    //         None
    //     };

    //     let mut include_columns: Vec<_> = self
    //         .components
    //         .iter_mut()
    //         .filter(|c| include.contains(&c.id))
    //         .collect();

    //     // TODO(anissen): Could split in (first, rest) for optimization
    //     if let Some(first) = include_columns.first() {
    //         let mut intersection = first.bitset.clone();
    //         include_columns
    //             .iter()
    //             .map(|col| &col.bitset)
    //             .for_each(|bitset| intersection.intersect_with(bitset));

    //         if let Some(exclude_bitset) = exclude_bitset {
    //             intersection.disjoint_with(&exclude_bitset);
    //         }

    //         intersection.iter_ids().for_each(|entity| {
    //             let mut row: Vec<_> = include_columns
    //                 .iter_mut()
    //                 .flat_map(|col| col.get_mut(entity))
    //                 .collect();
    //             system(entity, &mut row);
    //         });
    //     }
    // }

    // pub fn query_mut<'a>(
    //     &'a mut self,
    //     include: &Vec<ComponentId>,
    //     exclude: &Vec<ComponentId>,
    // ) -> QueryResultMutIter<'a> {
    //     // ) -> Vec<(Entity, Vec<&mut [u8]>)> {
    //     if include.is_empty() {
    //         return QueryResultMutIter::empty();
    //     }

    //     let exclude_columns: Vec<_> = self
    //         .components
    //         .iter()
    //         .filter(|c| exclude.contains(&c.id))
    //         .collect();

    //     // TODO(anissen): Could split in (first, rest) for optimization
    //     let exclude_bitset = if let Some(first) = exclude_columns.first() {
    //         let mut bitset = first.bitset.clone();
    //         self.components
    //             .iter()
    //             .filter(|c| exclude.contains(&c.id))
    //             .map(|col| &col.bitset)
    //             .for_each(|other| bitset.intersect_with(other));
    //         Some(bitset)
    //     } else {
    //         None
    //     };

    //     let mut include_columns: Vec<_> = self
    //         .components
    //         .iter_mut()
    //         .filter(|c| include.contains(&c.id))
    //         .collect();

    //     // TODO(anissen): Could split in (first, rest) for optimization
    //     if let Some(first) = include_columns.first() {
    //         let mut intersection = first.bitset.clone();
    //         include_columns
    //             .iter()
    //             .map(|col| &col.bitset)
    //             .for_each(|bitset| intersection.intersect_with(bitset));

    //         if let Some(exclude_bitset) = exclude_bitset {
    //             intersection.disjoint_with(&exclude_bitset);
    //         }

    //         let entities: Vec<_> = intersection.iter_ids().collect();
    //         let mut result = Vec::new();

    //         // SAFETY: We use raw pointers here to work around the borrow checker.
    //         // This is safe because:
    //         // 1. Each entity accesses a unique memory location within each column
    //         // 2. We never access the same memory location twice in the same iteration
    //         // 3. The pointers remain valid for the lifetime 'a (tied to &'a mut self)
    //         let column_ptrs: Vec<*mut Column> = include_columns
    //             .iter_mut()
    //             .map(|col| *col as *mut Column)
    //             .collect();

    //         for entity in entities {
    //             let mut row = Vec::new();
    //             for &col_ptr in &column_ptrs {
    //                 unsafe {
    //                     if let Some(data) = (*col_ptr).get_mut(entity) {
    //                         row.push(data);
    //                     }
    //                 }
    //             }
    //             result.push((entity, row));
    //         }

    //         QueryResultMutIter::new(result)
    //     } else {
    //         QueryResultMutIter::empty()
    //     }
    // }

    pub fn query(&mut self, include: &Vec<ComponentId>, exclude: &Vec<ComponentId>) -> QueryResult {
        let exclude_columns = self
            .components
            .iter()
            .filter(|c| exclude.contains(&c.id))
            .collect::<Vec<_>>();

        let exclude_bitmap = if let Some((first, rest)) = exclude_columns.split_first() {
            let mut bitset = first.bitset.clone();
            for col in rest {
                bitset.intersect_with(&col.bitset);
            }
            bitset
        } else {
            BitSet::new_empty(0)
        };

        let include_columns = self
            .components
            .iter_mut()
            .filter(|c| include.contains(&c.id))
            .collect::<Vec<_>>();

        let matching_entities = if let Some((first, rest)) = include_columns.split_first() {
            let mut bitset = first.bitset.clone();
            for col in rest {
                bitset.intersect_with(&col.bitset);
            }
            bitset.disjoint_with(&exclude_bitmap);

            // bitset.iter_ids().collect::<Vec<_>>()
            bitset.collect_set()
        } else {
            Vec::new()
        };

        if !matching_entities.is_empty() {
            // let non_marker_include_columns = include_columns
            //     .into_iter()
            //     .filter(|c| c.layout.size != 0)
            //     .collect();
            let non_marker_include_column_ids = include_columns
                .into_iter()
                .filter(|c| c.layout.size != 0)
                .map(|c| c.id)
                .collect();
            QueryResult::new(matching_entities, non_marker_include_column_ids)
        } else {
            QueryResult::new(Vec::new(), Vec::new())
        }
    }
}
