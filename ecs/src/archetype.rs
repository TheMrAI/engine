use core::alloc;
use std::ptr;

use crate::{Blob, ComponentDropFn, blob};

#[derive(Debug, Default)]
pub struct Archetype {
    size: usize,
    capacity: usize,
    components: Vec<Blob>,
    drop_fns: Vec<Option<ComponentDropFn>>,
}

impl Archetype {
    pub fn with_capacity(
        capacity: usize,
        component_layouts: Vec<alloc::Layout>,
        component_drops: Vec<Option<ComponentDropFn>>,
    ) -> Self {
        debug_assert!(
            component_layouts.len() == component_drops.len(),
            "The number of layouts and drop functions must match"
        );

        let components = component_layouts
            .into_iter()
            .map(|item_layout| Blob::with_capacity(item_layout, capacity))
            .collect();

        Self {
            size: 0,
            capacity,
            components,
            drop_fns: component_drops,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn push(&mut self, components: &[ptr::NonNull<u8>]) -> usize {
        debug_assert!(
            self.components.len() == components.len(),
            "The number of components must exactly match the number of components stored in the \
             Archetype"
        );

        // TODO asserts n shit
        self.ensure_capacity();
        for (i, component) in components.iter().enumerate() {
            unsafe {
                self.components[i].set(self.size, *component);
            }
        }
        let index = self.size;
        self.size += 1;
        index
    }

    pub fn pop(&mut self) {
        // TODO asserts n shit
        let entity_index = self.size - 1;
        for (i, blob) in self.components.iter_mut().enumerate() {
            unsafe { blob.swap_remove(entity_index, self.size, self.drop_fns[i]) };
        }
        self.size -= 1;
    }

    // Danger zone, this removes the value from the blob.
    // Releasing it properly is now the job of the caller.
    pub fn swap_remove(&mut self, entity_index: usize) {
        for (i, blob) in self.components.iter_mut().enumerate() {
            unsafe { blob.swap_remove(entity_index, self.size, self.drop_fns[i]) };
        }
        self.size -= 1;
    }

    pub fn get_component(&self, component_index: usize, entry_row: usize) -> ptr::NonNull<u8> {
        unsafe { self.components[component_index].get(entry_row) }
    }

    pub fn get_entity(&self, entry_row: usize) -> Vec<ptr::NonNull<u8>> {
        self.components
            .iter()
            .map(|blob| unsafe { blob.get(entry_row) })
            .collect()
    }

    pub fn get_column_iter<'archetype, T: 'static>(
        &'archetype self,
        column: usize,
    ) -> blob::BlobIterator<'archetype, T> {
        unsafe { self.components[column].iter::<T>(self.size) }
    }

    fn ensure_capacity(&mut self) {
        if self.size != self.capacity {
            return;
        }
        let capacity = if self.capacity == 0 { 1 } else { self.capacity };
        let new_capacity = capacity * 2;
        for blob in self.components.iter_mut() {
            unsafe {
                blob.grow(self.size, self.capacity, new_capacity);
            }
        }
        self.capacity = new_capacity;
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        for (i, component) in self.components.iter_mut().enumerate() {
            unsafe {
                (*component).drop(self.size, self.capacity, self.drop_fns[i]);
            }
        }
    }
}

#[cfg(test)]
mod no_drop_needed {
    use std::{alloc, ptr::NonNull};

    use crate::{ComponentDropFn, archetype::Archetype};

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    struct ComponentA {
        a: usize,
        b: bool,
    }

    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    struct ComponentB {
        a: i32,
        b: u32,
        c: i64,
    }

    const TEST_LAYOUT: [alloc::Layout; 3] = [
        alloc::Layout::new::<ComponentA>(),
        alloc::Layout::new::<ComponentB>(),
        alloc::Layout::new::<u8>(),
    ];
    const TEST_DROP_FNS: [Option<ComponentDropFn>; 3] = [None, None, None];
    const TEST_ENTITIES: [(ComponentA, ComponentB, u8); 3] = [
        (
            ComponentA { a: 2, b: true },
            ComponentB {
                a: -3223,
                b: 788887,
                c: -788887,
            },
            0,
        ),
        (
            ComponentA {
                a: 11111111,
                b: false,
            },
            ComponentB {
                a: -2332,
                b: 877778,
                c: -877778,
            },
            1,
        ),
        (
            ComponentA {
                a: 9999999999,
                b: true,
            },
            ComponentB {
                a: -3223,
                b: 666666,
                c: -666666,
            },
            2,
        ),
    ];

    fn insert_test_entity(archetype: &mut Archetype, test_entity_index: usize) -> usize {
        let mut entry = TEST_ENTITIES[test_entity_index];
        let component_a_ptr = unsafe { NonNull::new_unchecked(&mut entry.0 as *mut _).cast() };
        let component_b_ptr = unsafe { NonNull::new_unchecked(&mut entry.1 as *mut _).cast() };
        let component_c_ptr = unsafe { NonNull::new_unchecked(&mut entry.2 as *mut _).cast() };

        archetype.push(&[component_a_ptr, component_b_ptr, component_c_ptr])
    }

    fn compare_with_test_entity(
        archetype: &Archetype,
        entity_index: usize,
        test_entity_index: usize,
    ) {
        let entity_components = archetype.get_entity(entity_index);
        let entity_component_a = entity_components[0].cast::<ComponentA>();
        let entity_component_b = entity_components[1].cast::<ComponentB>();
        let entity_component_c = entity_components[2].cast::<u8>();

        assert_eq!(
            unsafe { *entity_component_a.as_ref() },
            TEST_ENTITIES[test_entity_index].0
        );
        assert_eq!(
            unsafe { *entity_component_b.as_ref() },
            TEST_ENTITIES[test_entity_index].1
        );
        assert_eq!(
            unsafe { *entity_component_c.as_ref() },
            TEST_ENTITIES[test_entity_index].2
        );
    }

    #[test]
    fn create() {
        let archetype = Archetype::with_capacity(5, TEST_LAYOUT.into(), TEST_DROP_FNS.into());

        assert_eq!(archetype.len(), 0);
        assert_eq!(archetype.capacity(), 5);
    }

    #[test]
    fn add_entries() {
        let mut archetype = Archetype::with_capacity(0, TEST_LAYOUT.into(), TEST_DROP_FNS.into());

        assert_eq!(archetype.len(), 0);
        assert_eq!(archetype.capacity(), 0);

        let entity_0 = insert_test_entity(&mut archetype, 0);

        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.capacity(), 2);

        let entity_1 = insert_test_entity(&mut archetype, 1);

        assert_eq!(archetype.len(), 2);
        assert_eq!(archetype.capacity(), 2);

        let entity_2 = insert_test_entity(&mut archetype, 2);

        assert_eq!(archetype.len(), 3);
        assert_eq!(archetype.capacity(), 4);

        compare_with_test_entity(&archetype, entity_0, 0);
        compare_with_test_entity(&archetype, entity_1, 1);
        compare_with_test_entity(&archetype, entity_2, 2);
    }

    #[test]
    fn pop_entries() {
        let mut archetype = Archetype::with_capacity(0, TEST_LAYOUT.into(), TEST_DROP_FNS.into());

        let entity_0 = insert_test_entity(&mut archetype, 0);
        let entity_1 = insert_test_entity(&mut archetype, 1);
        insert_test_entity(&mut archetype, 2);

        assert_eq!(archetype.len(), 3);
        assert_eq!(archetype.capacity(), 4);

        archetype.pop();

        assert_eq!(archetype.len(), 2);
        assert_eq!(archetype.capacity(), 4);
        compare_with_test_entity(&archetype, entity_0, 0);
        compare_with_test_entity(&archetype, entity_1, 1);

        archetype.pop();

        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.capacity(), 4);
        compare_with_test_entity(&archetype, entity_0, 0);

        archetype.pop();

        assert_eq!(archetype.len(), 0);
        assert_eq!(archetype.capacity(), 4);
    }

    #[test]
    fn swap_remove_entries() {
        let mut archetype = Archetype::with_capacity(0, TEST_LAYOUT.into(), TEST_DROP_FNS.into());

        insert_test_entity(&mut archetype, 0);
        insert_test_entity(&mut archetype, 1);
        insert_test_entity(&mut archetype, 2);

        assert_eq!(archetype.len(), 3);
        assert_eq!(archetype.capacity(), 4);

        archetype.swap_remove(0);

        assert_eq!(archetype.len(), 2);
        assert_eq!(archetype.capacity(), 4);
        compare_with_test_entity(&archetype, 0, 2);
        compare_with_test_entity(&archetype, 1, 1);

        archetype.swap_remove(0);

        assert_eq!(archetype.len(), 1);
        assert_eq!(archetype.capacity(), 4);
        compare_with_test_entity(&archetype, 0, 1);

        archetype.swap_remove(0);

        assert_eq!(archetype.len(), 0);
        assert_eq!(archetype.capacity(), 4);
    }
}
