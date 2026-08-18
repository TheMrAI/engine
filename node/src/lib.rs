use std::{alloc, ptr};

use lina::matrix::Matrix;

pub trait Node {
    fn create(world: &mut ecs::World) -> Self;
    fn entity_id(&self) -> ecs::EntityId;
}

// pub trait SpatialNode: Node {
//     fn parent(&self) -> Option<ecs::EntityId>;
//     fn children(&self) -> &[ecs::EntityId];
// }

#[derive(Debug, PartialEq)]
struct Geometry {
    mesh_id: u32,
}

struct Position {
    model_matrix: Matrix<f32, 4, 4>,
}

struct BasicNode {
    entity_id: ecs::EntityId,
    // parent: Option<ecs::EntityId>,
    // children: Vec<ecs::EntityId>,
}

impl Node for BasicNode {
    fn create(world: &mut ecs::World) -> Self {
        let mut geometry_component = Geometry { mesh_id: 3 };
        let mut position_component = Position {
            model_matrix: graphic::identity_matrix(),
        };

        let geometry_component_id = world.register_component::<Geometry>();
        let position_component_id = world.register_component::<Position>();

        // Component parts
        let mut component_entries: Vec<(
            u32,
            alloc::Layout,
            Option<ecs::ComponentDropFn>,
            ptr::NonNull<u8>,
        )> = vec![
            (
                geometry_component_id,
                alloc::Layout::new::<Geometry>(),
                None,
                unsafe {
                    ptr::NonNull::new_unchecked(&mut geometry_component as *mut Geometry).cast()
                },
            ),
            (
                position_component_id,
                alloc::Layout::new::<Position>(),
                None,
                unsafe {
                    ptr::NonNull::new_unchecked(&mut position_component as *mut Position).cast()
                },
            ),
        ];
        component_entries.sort_by_key(|lhs| lhs.0);
        let entity_data = {
            let mut ids = Vec::<ecs::ComponentId>::new();
            let mut layouts = Vec::<alloc::Layout>::new();
            let mut drop_fns = Vec::<Option<ecs::ComponentDropFn>>::new();
            let mut data = Vec::<ptr::NonNull<u8>>::new();

            for (id, layout, drop_fn, value) in component_entries.into_iter() {
                ids.push(id);
                layouts.push(layout);
                drop_fns.push(drop_fn);
                data.push(value);
            }

            (ids, layouts, drop_fns, data)
        };
        world.add_entity(0, entity_data);

        Self { entity_id: 0 }
    }

    fn entity_id(&self) -> ecs::EntityId {
        self.entity_id
    }
}

struct GeometryNode {
    entity_id: ecs::EntityId,
}

impl Node for GeometryNode {
    fn create(world: &mut ecs::World) -> Self {
        let mut geometry_component = Geometry { mesh_id: 1 };

        let geometry_component_id = world.register_component::<Geometry>();

        // Component parts
        let mut component_entries: Vec<(
            u32,
            alloc::Layout,
            Option<ecs::ComponentDropFn>,
            ptr::NonNull<u8>,
        )> = vec![(
            geometry_component_id,
            alloc::Layout::new::<Geometry>(),
            None,
            unsafe { ptr::NonNull::new_unchecked(&mut geometry_component as *mut Geometry).cast() },
        )];
        component_entries.sort_by_key(|lhs| lhs.0);
        let entity_data = {
            let mut ids = Vec::<ecs::ComponentId>::new();
            let mut layouts = Vec::<alloc::Layout>::new();
            let mut drop_fns = Vec::<Option<ecs::ComponentDropFn>>::new();
            let mut data = Vec::<ptr::NonNull<u8>>::new();

            for (id, layout, drop_fn, value) in component_entries.into_iter() {
                ids.push(id);
                layouts.push(layout);
                drop_fns.push(drop_fn);
                data.push(value);
            }

            (ids, layouts, drop_fns, data)
        };
        world.add_entity(1, entity_data);

        Self { entity_id: 1 }
    }

    fn entity_id(&self) -> ecs::EntityId {
        self.entity_id
    }
}

#[cfg(test)]
mod tests {
    use crate::{BasicNode, Geometry, GeometryNode, Node};

    #[test]
    fn quick_check() {
        let mut world = ecs::World::default();

        let basic_node = BasicNode::create(&mut world);
        assert_eq!(basic_node.entity_id(), 0);
        let geometry_node = GeometryNode::create(&mut world);
        assert_eq!(geometry_node.entity_id(), 1);

        let geometry_iter = world.query::<Geometry>();
        let mut values = geometry_iter.map(|node| node.mesh_id).collect::<Vec<u32>>();
        values.sort();
        assert_eq!(values.as_slice(), [1, 3]);
    }
}
