use std::{alloc, cell::RefCell, collections::HashMap, ptr::NonNull, rc::Rc};

use crate::{blob::Blob, signature::DynamicSignature};

mod blob;
pub mod signature;

type EntityId = u32;
type ArchetypeId = u32;
type ComponentId = u32;
type ComponentLayouts = Vec<alloc::Layout>;
type ArchetypeMap = HashMap<ArchetypeId, ArchetypeRecord>;
type SharedArchetype = Rc<RefCell<Archetype>>;

#[derive(Debug, Default)]
pub struct Archetype {
    id: ArchetypeId,
    components: Vec<Blob>,
    capacity: usize,
    size: usize,
}

impl Drop for Archetype {
    fn drop(&mut self) {
        for component in &mut self.components {
            unsafe {
                component.drop(self.size, self.capacity, None);
            }
        }
    }
}

#[derive(Debug, Default)]
struct ArchetypeRecord {
    column: usize,
}

#[derive(Debug, Default)]
struct Record {
    archetype: SharedArchetype,
    row: usize,
}

#[derive(Debug, Default)]
pub struct World {
    entity_index: HashMap<EntityId, Record>,
    archetype_index: HashMap<DynamicSignature, SharedArchetype>,
    component_index: HashMap<ComponentId, ArchetypeMap>,
}

impl World {
    pub fn has_component(&self, entity_id: EntityId, component_id: ComponentId) -> bool {
        let record = &self.entity_index[&entity_id];
        let archetype_id = record.archetype.borrow().id;
        let archetype_map = match self.component_index.get(&component_id) {
            Some(archetype_set) => archetype_set,
            None => return false,
        };
        archetype_map.contains_key(&archetype_id)
    }

    pub fn get_component(&self, entity_id: EntityId, component_id: ComponentId) -> NonNull<u8> {
        let record = &self.entity_index[&entity_id];
        let archetype = &record.archetype;

        let archetype_map = self.component_index.get(&component_id).unwrap();

        let archetype_record = match archetype_map.get(&archetype.borrow().id) {
            Some(record) => record,
            None => unreachable!("Nope"),
        };
        archetype.borrow().components[archetype_record.column].get_item(record.row)
    }

    pub fn add_entity(
        &mut self,
        entity_id: EntityId,
        entity_type: DynamicSignature,
        component_layouts: ComponentLayouts,
    ) {
        let archetype = self
            .archetype_index
            .entry(entity_type.clone())
            .or_insert_with(|| {
                let capacity = 5;
                let components = component_layouts
                    .into_iter()
                    .map(|component_layout| Blob::with_capacity(component_layout, capacity))
                    .collect();

                Rc::new(RefCell::new(Archetype {
                    id: 0,
                    capacity,
                    size: 0,
                    components,
                }))
            })
            .clone();

        let index = archetype.borrow().size;
        archetype.borrow_mut().size += 1;
        let record = Record {
            archetype: archetype.clone(),
            row: index,
        };
        // testing hack
        let ptr = core::ptr::NonNull::new((&mut 3u16 as *mut u16).cast::<u8>()).unwrap();
        archetype.borrow_mut().components[0].place_at(ptr, 0);

        self.entity_index.insert(entity_id, record);

        for component in entity_type.components() {
            self.component_index
                .entry(*component)
                .and_modify(|archetype_map| {
                    archetype_map.insert(0, ArchetypeRecord { column: 0 });
                })
                .or_insert({
                    let mut archetype_map = ArchetypeMap::new();
                    archetype_map.insert(0, ArchetypeRecord { column: 0 });
                    archetype_map
                });
        }
    }
}

#[cfg(test)]
mod tests {
    // use std::alloc;

    // use crate::{World, signature::DynamicSignature};

    // #[test]
    // fn has() {
    //     let mut world = World::default();
    //     world.add_entity(
    //         0,
    //         DynamicSignature::from_slice(&[1]),
    //         vec![alloc::Layout::new::<u16>()],
    //     );
    //     println!("Has checking");
    //     let expected = [false, true, false, false, false];
    //     for i in 0..5 {
    //         assert_eq!(world.has_component(0, i), expected[i as usize])
    //     }

    //     assert_eq!(unsafe { world.get_component(0, 1).cast::<u16>().read() }, 3);
    // }
}
