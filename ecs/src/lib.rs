use std::{alloc, cell::RefCell, collections::HashMap, ptr, rc::Rc};

use crate::{archetype::Archetype, blob::Blob, signature::DynamicSignature};

pub mod archetype;
mod blob;
pub mod signature;

type ComponentDropFn = unsafe fn(ptr::NonNull<u8>) -> ();
type EntityId = u32;
type ArchetypeId = u32;
type ComponentId = u32;
type ComponentLayouts = Vec<alloc::Layout>;
type ArchetypeMap = HashMap<ArchetypeId, ArchetypeRecord>;
type SharedArchetype = Rc<RefCell<Archetype>>;

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
        let archetype_id = record.archetype.borrow().id();
        let archetype_map = match self.component_index.get(&component_id) {
            Some(archetype_set) => archetype_set,
            None => return false,
        };
        archetype_map.contains_key(&archetype_id)
    }

    pub fn get_component(
        &self,
        entity_id: EntityId,
        component_id: ComponentId,
    ) -> ptr::NonNull<u8> {
        let record = &self.entity_index[&entity_id];
        let archetype = &record.archetype;

        let archetype_map = self.component_index.get(&component_id).unwrap();

        let archetype_record = match archetype_map.get(&archetype.borrow().id()) {
            Some(record) => record,
            None => unreachable!("Nope"),
        };
        archetype
            .borrow()
            .get_component(archetype_record.column, record.row)
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
                Rc::new(RefCell::new(Archetype::with_capacity(
                    0,
                    5,
                    component_layouts,
                    vec![None],
                )))
            })
            .clone();

        // testing hack
        let mut val = 3u16;
        let ptr = ptr::NonNull::new((&mut val as *mut u16).cast::<u8>()).unwrap();
        let index = archetype.borrow_mut().push(&[ptr]);
        let record = Record {
            archetype: archetype.clone(),
            row: index,
        };

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
    use std::alloc;

    use crate::{World, signature::DynamicSignature};

    #[test]
    fn has() {
        let mut world = World::default();
        world.add_entity(
            0,
            DynamicSignature::from_slice(&[1]),
            vec![alloc::Layout::new::<u16>()],
        );
        println!("Has checking");
        let expected = [false, true, false, false, false];
        for i in 0..5 {
            assert_eq!(world.has_component(0, i), expected[i as usize])
        }

        assert_eq!(unsafe { world.get_component(0, 1).cast::<u16>().read() }, 3);
    }
}
