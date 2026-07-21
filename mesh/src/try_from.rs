use format::wavefront;

use crate::{Mesh, Vertex};

impl std::convert::TryFrom<wavefront::Obj> for Mesh {
    // TODO handle errors
    type Error = u8;

    fn try_from(obj: wavefront::Obj) -> Result<Self, Self::Error> {
        let vertices = obj
            .faces()
            .iter()
            .flat_map(|face| {
                face.map(|obj_vertex| {
                    let vertices = obj.vertices();
                    let normals = obj.normals();
                    let uvs = obj.uv_coords();

                    let position = vertices[obj_vertex.vertex_index() - 1].into();
                    let normal = normals[obj_vertex.normal_index().unwrap() - 1].into();
                    let uv_coords = match obj_vertex.uv_index() {
                        Some(uv_index) => uvs[uv_index - 1],
                        None => [0.0, 0.0, 0.0],
                    };
                    let uv = [uv_coords[0], uv_coords[1]].into();

                    Vertex {
                        position,
                        normal,
                        uv,
                    }
                })
            })
            .collect::<Vec<Vertex>>();
        let indices = (0..vertices.len() as u32).collect();

        Ok(Mesh { vertices, indices })
    }
}
