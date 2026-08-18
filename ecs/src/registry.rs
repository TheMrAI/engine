use std::{
    any::{self},
    collections::HashMap,
};

/// Registry for component types
///
/// The [any::TypeId] of the component type will be used
/// to identify a type. This is then mapped onto a simple [u32] id.
///
/// The format or the value of the [any::TypeId] is unstable, but
/// that is okay as we do not rely on it.
/// [any::TypeId] only exists for types that
/// adhere to `'static` lifetime. This means that there may be no
/// non-static reference within the type itself.
/// In terms of the logic of the ECS this should be okay, as the goal
/// is to store data directly in continuous memory. It is directly
/// detrimental to performance if additional indirections are present
/// in the stored components.
#[derive(Default, Debug)]
pub struct ComponentRegistry {
    components: HashMap<any::TypeId, u32>,
    next_id: u32,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, type_id: any::TypeId) -> Option<&u32> {
        self.components.get(&type_id)
    }

    pub fn get_or_insert_id(&mut self, type_id: any::TypeId) -> u32 {
        *self.components.entry(type_id).or_insert_with(|| {
            let id = self.next_id;
            self.next_id += 1;
            id
        })
    }
}
