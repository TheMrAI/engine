use std::{
    alloc, any,
    collections::HashMap,
    ptr::{self, NonNull},
};

use cyclid::Cyclid;

use crate::{archetype::Archetype, blob::Blob, registry::ComponentRegistry};

pub mod archetype;
mod blob;
mod registry;
pub mod signature;

pub type ComponentDropFn = unsafe fn(ptr::NonNull<u8>) -> ();
pub type EntityId = u32;
type ArchetypeId = usize;
pub type ComponentId = u32;
type ComponentLayouts = Vec<alloc::Layout>;
type ComponentValues = Vec<NonNull<u8>>;
type ArchetypeMap = HashMap<ArchetypeId, ArchetypeRecord>;

#[derive(Debug, Default)]
struct ArchetypeRecord {
    column: usize,
}

#[derive(Debug, Default)]
struct Record {
    archetype_id: ArchetypeId,
    row: usize,
}

#[derive(Debug, Default)]
pub struct World {
    entity_index: HashMap<EntityId, Record>,
    archetype_index: HashMap<Vec<ComponentId>, ArchetypeId>,
    component_index: HashMap<ComponentId, ArchetypeMap>,
    archetypes: Cyclid<Archetype>,
    component_registry: ComponentRegistry,
}

impl World {
    pub fn has_component(&self, entity_id: EntityId, component_id: ComponentId) -> bool {
        let record = &self.entity_index[&entity_id];
        let archetype_id = record.archetype_id;
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
        let archetype_id = record.archetype_id;
        let archetype = self.archetypes.get(archetype_id);

        let archetype_map = self.component_index.get(&component_id).unwrap();

        let archetype_record = match archetype_map.get(&archetype_id) {
            Some(record) => record,
            None => unreachable!("Nope"),
        };
        archetype.get_component(archetype_record.column, record.row)
    }

    /// Add an entity to the world
    ///
    /// The function is a bit disturbing at the moment.
    /// [EntityId] is passed in, and it must not collide with other
    /// existing entities.
    /// component_data is a tuple of vectors that must be properly ordered
    /// in sync with [ComponentId].
    /// The ordering is critically important as it uniquely identifies the
    /// [Archetype].
    /// TODO: need a design where this can't be messed up
    pub fn add_entity(
        &mut self,
        entity_id: EntityId,
        component_data: (
            Vec<ComponentId>,
            ComponentLayouts,
            Vec<Option<ComponentDropFn>>,
            ComponentValues,
        ),
    ) {
        let archetype_id = self
            .archetype_index
            .entry(component_data.0.clone())
            .or_insert_with(|| {
                self.archetypes.insert(Archetype::with_capacity(
                    5,
                    component_data.1,
                    component_data.2,
                ))
            });

        let archetype = self.archetypes.get_mut(*archetype_id);

        let index = archetype.push(&component_data.3);
        let record = Record {
            archetype_id: *archetype_id,
            row: index,
        };

        self.entity_index.insert(entity_id, record);

        for (i, component) in component_data.0.iter().enumerate() {
            self.component_index
                .entry(*component)
                .and_modify(|archetype_map| {
                    archetype_map.insert(*archetype_id, ArchetypeRecord { column: i });
                })
                .or_insert({
                    let mut archetype_map = ArchetypeMap::new();
                    archetype_map.insert(*archetype_id, ArchetypeRecord { column: i });
                    archetype_map
                });
        }
    }

    pub fn register_component<T: 'static>(&mut self) -> ComponentId {
        self.component_registry
            .get_or_insert_id(any::TypeId::of::<T>())
    }

    pub fn query<T: 'static>(&self) -> impl Iterator<Item = &T> {
        let component_id = self.component_registry.get(any::TypeId::of::<T>()).unwrap();

        let archetype_map = self.component_index.get(component_id).unwrap();
        archetype_map
            .iter()
            .flat_map(|(key, val)| self.archetypes.get(*key).get_column_iter(val.column))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        alloc,
        ptr::{self, NonNull},
    };

    use crate::{ComponentDropFn, ComponentId, World};

    #[derive(Debug, PartialEq)]
    struct ComponentA {
        a: usize,
    }

    #[derive(Debug, PartialEq)]
    struct ComponentB {
        a: i32,
        b: u8,
        c: u64,
    }

    #[test]
    fn add_component() {
        let mut world = World::default();
        let mut component_a = ComponentA { a: 988888889 };
        let mut component_b = ComponentB {
            a: -33333,
            b: 12,
            c: 1234567890,
        };

        let component_a_id = world.register_component::<ComponentA>();
        let component_b_id = world.register_component::<ComponentB>();

        // Component parts
        let mut component_entries: Vec<(u32, alloc::Layout, Option<ComponentDropFn>, NonNull<u8>)> = vec![
            (
                component_a_id,
                alloc::Layout::new::<ComponentA>(),
                None,
                unsafe { ptr::NonNull::new_unchecked(&mut component_a as *mut ComponentA).cast() },
            ),
            (
                component_b_id,
                alloc::Layout::new::<ComponentB>(),
                None,
                unsafe { ptr::NonNull::new_unchecked(&mut component_b as *mut ComponentB).cast() },
            ),
        ];
        component_entries.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
        let entity_data = {
            let mut ids = Vec::<ComponentId>::new();
            let mut layouts = Vec::<alloc::Layout>::new();
            let mut drop_fns = Vec::<Option<ComponentDropFn>>::new();
            let mut data = Vec::<NonNull<u8>>::new();

            for (id, layout, drop_fn, value) in component_entries.into_iter() {
                ids.push(id);
                layouts.push(layout);
                drop_fns.push(drop_fn);
                data.push(value);
            }

            (ids, layouts, drop_fns, data)
        };
        world.add_entity(0, entity_data);

        let stored_component_a = world.get_component(0, component_a_id).cast::<ComponentA>();
        let stored_component_b = world.get_component(0, component_b_id).cast::<ComponentB>();

        assert_eq!(unsafe { stored_component_a.as_ref().a }, component_a.a);
        assert_eq!(unsafe { stored_component_b.as_ref().a }, component_b.a);
        assert_eq!(unsafe { stored_component_b.as_ref().b }, component_b.b);
        assert_eq!(unsafe { stored_component_b.as_ref().c }, component_b.c);

        assert_eq!(&component_a, unsafe { stored_component_a.as_ref() });
        assert_eq!(&component_b, unsafe { stored_component_b.as_ref() });
    }
}
