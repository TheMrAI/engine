use lina::matrix::Matrix;

// This is intended to be part of the Scene graph.
// As it can be seen it should not contain much data at all.
// Simply because most of it isn't and shouldn't be owned by it.
// The Engine is not concerned by what mesh, texture, shader etc is
// used for a node to render it. It shouldn't even know whether or not it
// should be rendered at all.
//
// TODO this is a very rudimentary interface, just to test if we are moving in
// the right direction.
#[derive(Debug, Clone, Copy)]
pub struct MeshNode {
    pub mesh_id: u32,
    pub model_matrix: Matrix<f32, 4, 4>,
    pub shader_id: u32,
    // These are 'material'/'shader' related properties
    // not sure how to sequester them yet
    pub texture_id: Option<u32>,
    pub texture_scale: Option<f32>,
}
