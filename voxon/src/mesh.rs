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
pub fn generate_cube() -> Mesh {
    // Vertex buffer
    #[rustfmt::skip]
    let vertex_positions: Vec<Vector<f32, 4>> = vec![
        // front
        v![-1.0, -1.0, 1.0, 1.0],
        v![1.0, -1.0, 1.0, 1.0],
        v![1.0, 1.0, 1.0, 1.0],
        v![-1.0, 1.0, 1.0, 1.0],
        // right
        v![1.0, -1.0, 1.0, 1.0],
        v![1.0, -1.0, -1.0, 1.0],
        v![1.0, 1.0, -1.0, 1.0],
        v![1.0, 1.0, 1.0, 1.0],
        // back
        v![1.0, -1.0, -1.0, 1.0],
        v![-1.0, -1.0, -1.0, 1.0],
        v![-1.0, 1.0, -1.0, 1.0],
        v![1.0, 1.0, -1.0, 1.0],
        // left
        v![-1.0, -1.0, -1.0, 1.0],
        v![-1.0, -1.0, 1.0, 1.0],
        v![-1.0, 1.0, 1.0, 1.0],
        v![-1.0, 1.0, -1.0, 1.0],
        // top
        v![-1.0, 1.0, -1.0, 1.0],
        v![-1.0, 1.0, 1.0, 1.0],
        v![1.0, 1.0, 1.0, 1.0],
        v![1.0, 1.0, -1.0, 1.0],
        // bottom
        v![-1.0, -1.0, -1.0, 1.0],
        v![1.0, -1.0, -1.0, 1.0],
        v![1.0, -1.0, 1.0, 1.0],
        v![-1.0, -1.0, 1.0, 1.0],
    ];
    let normals: Vec<Vector<f32, 3>> = vec![
        // front
        v![0.0, 0.0, 1.0],
        // right
        v![1.0, 0.0, 0.0],
        // back
        v![0.0, 0.0, -1.0],
        // left
        v![-1.0, 0.0, 0.0],
        // top
        v![0.0, 1.0, 0.0],
        // bottom
        v![0.0, -1.0, 0.0],
    ];
    let vertices = vertex_positions
        .iter()
        .enumerate()
        .map(|(i, position)| Vertex {
            position: *position,
            normal: normals[i / 4],
        })
        .collect();

    // Vertex indices
    #[rustfmt::skip]
    let indices: Vec<u32> = vec![
        // front face
        0, 1, 2,
        2, 3, 0,
        // right face
        4, 5, 6,
        6, 7, 4,
        // back face
        8, 9, 10,
        10, 11, 8,
        // left face
        12, 13, 14,
        14, 15, 12,
        // top face
        16, 17, 18,
        18, 19, 16,
        // bottom face
        20, 21, 22,
        22, 23, 20,
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
