use lina::{v, vector::Vector};

pub struct Vertex {
    position: Vector<f32, 4>,
    normal: Vector<f32, 3>,
}

impl Vertex {
    pub fn position(&self) -> &Vector<f32, 4> {
        &self.position
    }

    pub fn normal(&self) -> &Vector<f32, 3> {
        &self.normal
    }
}

pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Mesh {
    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    pub fn indices(&self) -> &Vec<u32> {
        &self.indices
    }
}

/// The cube center is at (0, 0, 0) and has a dimensions
/// of 2.
/// Return a pair of vertices and their indexes.
pub fn generate_cube() -> Mesh {
    // Vertex buffer
    #[rustfmt::skip]
    let vertex_positions: Vec<Vector<f32, 4>> = vec![
        v![-1.0, -1.0, 1.0, 1.0], // 0
        v![1.0, -1.0, 1.0, 1.0], // 1
        v![1.0, 1.0, 1.0, 1.0], // 2
        v![-1.0, 1.0, 1.0, 1.0], // 3
        v![-1.0, -1.0, -1.0, 1.0], // 4
        v![1.0, -1.0, -1.0, 1.0], // 5
        v![1.0, 1.0, -1.0, 1.0], // 6
        v![-1.0, 1.0, -1.0, 1.0], // 7
    ];
    // The normal will be the same for each vertex as it's position,
    // normalized.
    let vertices = vertex_positions
        .iter()
        .map(|position| Vertex {
            position: *position,
            normal: position.xyz().unwrap().normalized(),
        })
        .collect();

    // Vertex indices
    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        // front face
        0, 1, 2,
        2, 3, 0,
        // back face
        5, 4, 7,
        7, 6, 5,
        // top face
        3, 2, 6,
        6, 7, 3,
        // bottom face
        4, 5, 1,
        1, 0, 4,
        // right face
        5, 6, 2,
        2, 1, 5,
        // left face
        4, 0, 3,
        3, 7, 4
    ];

    Mesh { vertices, indices }
}

/// A 2x2 big plane centered a the origo,
/// laying on the XZ plane.
pub fn generate_plane() -> Mesh {
    // Vertex buffer
    #[rustfmt::skip]
    let vertex_positions: Vec<Vector<f32, 4>> = vec![
        v![-1.0, 0.0, 1.0, 1.0], // 0
        v![1.0, 0.0, 1.0, 1.0], // 1
        v![1.0, 0.0, -1.0, 1.0], // 2
        v![-1.0, 0.0, -1.0, 1.0], // 3
    ];
    // The normal will be the same for each vertex, up.
    let vertices = vertex_positions
        .iter()
        .map(|position| Vertex {
            position: *position,
            normal: v![0.0, 1.0, 0.0],
        })
        .collect();

    // Vertex indices
    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        0, 1, 3,
        3, 1, 2  
    ];

    Mesh { vertices, indices }
}
