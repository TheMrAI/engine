//! Wavefront Object parser
//!
//! The format is ancient and there isn't an up
//! to date specification for it any more.
//! The closest you may find is: [Object Files (.obj)](https://paulbourke.net/dataformats/obj/)
//! This can lead to situations where different '.obj' exporters/importers have
//! interpreted the standard differently and may produce incompatible outcomes.
//!
//! Nothing to do about this, but in light of this, the below parser is kept
//! supporting only the minimal feature set, which is expected to function
//! everywhere the same.
//!
//! The object file has a binary format ('.mod'') and an ASCII format ('.obj')
//! as well. This library will not support the binary format. It is so uncommon,
//! that most people aren't even aware it exists. There is also a '.mtl' support
//! file for format storing materials data, but this won't be supported either.
//!
//! This object file parser is only meant for importing simple meshes. No
//! surfaces, material, raytracing or any other extensions are supported.

/// Parsed '.obj' representation
///
/// ## Format
///
/// As per '.obj' specification:
/// Vertices/uv coordinates/normals are all indexed from 1, not 0!
/// Coordinates use the right-hand coordinate system.
/// Faces do not have a specified order that they must follow. They may be in
/// clockwise or counter clockwise order. Only by their vertex normal may the
/// appropriate order be identified.
///
/// All vertex positions, uv coords and normals are added sequentially to their
/// respective global buffers. These buffers are global for the whole file.
/// Faces are defined by indexing into these buffers. While the standard
/// technically allows for using negative indexes for these, this parser does
/// not, as it unnecessarily complicates the format.
///
/// The file may include one or more objects/meshes. In either case the whole
/// [Obj] can be interpreted as a valid , single mesh, by trivially iterating
/// over all the faces and extracting the relevant vertex data.
#[derive(Debug, Default, Clone)]
pub struct Obj {
    global_name: String,
    vertices: Vec<[f32; 4]>,
    uv_coords: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    faces: Vec<[Vertex; 3]>,
    entries: Vec<NamedGroup>,
}

/// Each [Vertex] contains the index to the respective [Obj] buffers
///
/// From these three only the 'vertex position' is mandatory.
#[derive(Debug, Default, Clone, Copy)]
pub struct Vertex {
    vertex_index: usize,
    uv_index: Option<usize>,
    normal_index: Option<usize>,
}

impl Vertex {
    /// Get the index of the vertex's positional vector
    pub fn vertex_index(&self) -> usize {
        self.vertex_index
    }

    /// Get the index of the vertex's uv vector
    pub fn uv_index(&self) -> Option<usize> {
        self.uv_index
    }

    /// Get the index of the vertex's normal vector
    pub fn normal_index(&self) -> Option<usize> {
        self.normal_index
    }
}

/// Named group identifier
///
/// In case the [Obj] file contains named groups
/// specified by 'o group_name' entries in '.obj' format then these meshes
/// can be extracted individually from the [Obj] file using [NamedGroup]s.
#[derive(Debug, Default, Clone)]
pub struct NamedGroup {
    name: String,
    face_start_index: usize,
}

impl NamedGroup {
    /// The name of the group
    ///
    /// This can be interpreted as the name of the object/mesh.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Get the starting face index for this group
    ///
    /// Iterating the faces from this value, until the next [NamedGroup]'s
    /// starting index or the last face index if no other [NamedGroup] entry
    /// exists should provide all faces associated with this [NamedGroup].
    pub fn face_start_index(&self) -> usize {
        self.face_start_index
    }
}

impl Obj {
    /// The name of the [Obj]
    ///
    /// Regardless how many separate meshes may be in an [Obj] file
    /// , their whole collection may be identified as a single mesh.
    /// This mesh can be referenced with the `global name`. Otherwise
    /// the name serves no purpose.
    pub fn global_name(&self) -> &str {
        &self.global_name
    }

    /// The positional vectors for each vertex
    ///
    /// A vertex will have `x`, `y`, `z` and `w` coordinates
    /// set. `w` is optional, in which case it defaults to
    /// '0.0'.
    pub fn vertices(&self) -> &Vec<[f32; 4]> {
        &self.vertices
    }

    /// The texture coordinates for each vertex
    ///
    /// `u`, `v` and `w` components will always be set.
    /// `v` and `w` is optional, in which case it will have the
    /// value of '0.0'.
    /// The `.obj` file does not signal for the dimensionality of
    /// the required texture maps. We can only assume that if all
    /// values in a given dimension are set to the default value of '0.0', that
    /// that dimension is not expected to be used.
    /// For example if all `w` values are '0.0', but the rest aren't, a
    /// 2D texture is expected. If both `w` and `v` values are set to '0.0' then
    /// a 1D texture is expected. Otherwise a 3D texture is expected.
    pub fn uv_coords(&self) -> &Vec<[f32; 3]> {
        &self.uv_coords
    }

    /// The normal vectors for each vertex
    ///
    /// `i`, `j` and `k` components of the normal
    /// vector is always specified.
    pub fn normals(&self) -> &Vec<[f32; 3]> {
        &self.normals
    }

    /// The faces specified for the object
    ///
    /// '.obj' does not specify the order of the indexed components.
    /// It may be in `cw` or `ccw` order.
    ///
    /// A '.obj' file can technically specify Ngon faces, but this parser
    /// only supports regular triangles.
    /// This means that each face entry may only have 3 [Vertex] components.
    pub fn faces(&self) -> &Vec<[Vertex; 3]> {
        &self.faces
    }

    /// Return the [NamedGroup]
    pub fn named_groups(&self) -> &Vec<NamedGroup> {
        &self.entries
    }

    /// Parse the [Obj] from the given [String] input
    pub fn parse<Iter: Iterator<Item = String>>(lines: Iter, default_name: &str) -> Obj {
        let mut object = Obj {
            global_name: default_name.into(),
            ..Default::default()
        };

        for line in lines {
            if line.is_empty() {
                continue;
            }
            let (entry_type, entry_data) = line.split_once(" ").unwrap();
            // todo basic error checking
            match entry_type {
                "o" => {
                    object.entries.push(NamedGroup {
                        name: entry_data.trim().into(),
                        face_start_index: object.faces().len(),
                    })
                }
                "v" => {
                    let mut data = [0.0, 0.0, 0.0, 1.0];
                    parse_values(entry_data, &mut data);
                    object.vertices.push(data);
                }
                "vn" => {
                    let mut data = [0.0, 0.0, 0.0];
                    parse_values(entry_data, &mut data);
                    object.normals.push(data);
                }
                "vt" => {
                    let mut data = [0.0, 0.0, 0.0];
                    parse_values(entry_data, &mut data);
                    object.uv_coords.push(data);
                }
                "f" => {
                    let sections = entry_data.split(" ");
                    let face_data = sections.map(parse_vertex).collect::<Vec<Vertex>>();
                    // check if there are only 3 entries
                    object
                        .faces
                        .push([face_data[0], face_data[1], face_data[2]]);
                }
                "#" => continue,
                _ => continue,
            }
        }

        object
    }
}

fn parse_values<const MAX_COUNT: usize>(input: &str, data: &mut [f32; MAX_COUNT]) {
    input.split(" ").enumerate().for_each(|(i, value_str)| {
        data[i] = value_str.parse::<f32>().unwrap();
    });
}

fn parse_vertex(triplet_str: &str) -> Vertex {
    let mut vertex_index = 0;
    let mut uv_index = None;
    let mut normal_index = None;

    let values = triplet_str.split("/");
    values.enumerate().for_each(|(i, value_str)| {
        if value_str.is_empty() {
            return;
        }
        let value = value_str.parse::<usize>().unwrap();
        match i {
            0 => vertex_index = value,
            1 => uv_index = Some(value),
            2 => normal_index = Some(value),
            _ => unreachable!("A face entry may only have 3 components specified."),
        }
    });

    Vertex {
        vertex_index,
        uv_index,
        normal_index,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plane() {
        let plane = vec![
            "o Plane",
            "v -1.000000 0.000000 1.000000",
            "v 1.000000 0.000000 1.000000",
            "v -1.000000 0.000000 -1.000000",
            "v 1.000000 0.000000 -1.000000",
            "vn -0.0000 1.0000 -0.0000",
            "vt 1.000000 0.000000",
            "vt 0.000000 1.000000",
            "vt 0.000000 0.000000",
            "vt 1.000000 1.000000",
            "s 0",
            "f 2/1/1 3/2/1 1/3/1",
            "f 2/1/1 4/4/1 3/2/1",
        ];

        let object = Obj::parse(plane.into_iter().map(String::from), "global name");

        assert_eq!(object.global_name(), "global name");

        assert_eq!(object.vertices().len(), 4);
        assert_eq!(object.normals().len(), 1);
        assert_eq!(object.uv_coords().len(), 4);
        assert_eq!(object.faces().len(), 2);

        assert_eq!(object.named_groups().len(), 1);

        let named_group = &object.named_groups()[0];
        assert_eq!(named_group.name(), "Plane");
        assert_eq!(named_group.face_start_index(), 0);
    }

    #[test]
    fn parse_two_triangles() {
        let plane = vec![
            "o TriangleOne",
            "v -1.000000 0.000000 1.000000",
            "v 1.000000 0.000000 1.000000",
            "v -1.000000 0.000000 -1.000000",
            "vn -0.0000 1.0000 -0.0000",
            "vt 0.000000 0.000000",
            "vt 1.000000 0.000000",
            "vt 0.000000 1.000000",
            "s 0",
            "f 1/1/1 2/2/1 3/3/1",
            "o TriangleTwo",
            "v 0.939709 -0.082245 0.425844",
            "v -1.060290 -0.082245 -1.574156",
            "v 0.939709 -0.082245 -1.574156",
            "vn -0.0000 1.0000 -0.0000",
            "vt 1.000000 0.000000",
            "vt 1.000000 1.000000",
            "vt 0.000000 1.000000",
            "s 0",
            "f 4/4/2 6/5/2 5/6/2",
        ];

        let object = Obj::parse(plane.into_iter().map(String::from), "global name");

        assert_eq!(object.global_name(), "global name");
        assert_eq!(object.vertices().len(), 6);
        assert_eq!(object.normals().len(), 2);
        assert_eq!(object.uv_coords().len(), 6);
        assert_eq!(object.faces().len(), 2);

        assert_eq!(object.named_groups().len(), 2);

        let triangle_one = &object.named_groups()[0];
        assert_eq!(triangle_one.name(), "TriangleOne");
        assert_eq!(triangle_one.face_start_index(), 0);

        let triangle_two = &object.named_groups()[1];
        assert_eq!(triangle_two.name(), "TriangleTwo");
        assert_eq!(triangle_two.face_start_index(), 1);
    }

    #[test]
    fn parse_cube() {
        let cube = vec![
            "o Cube  ",
            "v 1.000000 1.000000 -1.000000",
            "v 1.000000 -1.000000 -1.000000",
            "v 1.000000 1.000000 1.000000",
            "v 1.000000 -1.000000 1.000000",
            "v -1.000000 1.000000 -1.000000",
            "v -1.000000 -1.000000 -1.000000",
            "v -1.000000 1.000000 1.000000",
            "v -1.000000 -1.000000 1.000000",
            "vn -0.0000 1.0000 -0.0000",
            "vn -0.0000 -0.0000 1.0000",
            "vn -1.0000 -0.0000 -0.0000",
            "vn -0.0000 -1.0000 -0.0000",
            "vn 1.0000 -0.0000 -0.0000",
            "vn -0.0000 -0.0000 -1.0000",
            "vt 0.875000 0.500000",
            "vt 0.625000 0.750000",
            "vt 0.625000 0.500000",
            "vt 0.375000 1.000000",
            "vt 0.375000 0.750000",
            "vt 0.625000 0.000000",
            "vt 0.375000 0.250000",
            "vt 0.375000 0.000000",
            "vt 0.375000 0.500000",
            "vt 0.125000 0.750000",
            "vt 0.125000 0.500000",
            "vt 0.625000 0.250000",
            "vt 0.875000 0.750000",
            "vt 0.625000 1.000000",
            "s 0",
            "f 5/1/1 3/2/1 1/3/1",
            "f 3/2/2 8/4/2 4/5/2",
            "f 7/6/3 6/7/3 8/8/3",
            "f 2/9/4 8/10/4 6/11/4",
            "f 1/3/5 4/5/5 2/9/5",
            "f 5/12/6 2/9/6 6/7/6",
            "f 5/1/1 7/13/1 3/2/1",
            "f 3/2/2 7/14/2 8/4/2",
            "f 7/6/3 5/12/3 6/7/3",
            "f 2/9/4 4/5/4 8/10/4",
            "f 1/3/5 3/2/5 4/5/5",
            "f 5/12/6 1/3/6 2/9/6",
        ];

        let object = Obj::parse(cube.into_iter().map(String::from), "global name");

        assert_eq!(object.global_name(), "global name");
        assert_eq!(object.vertices().len(), 8);
        assert_eq!(object.normals().len(), 6);
        assert_eq!(object.uv_coords().len(), 14);
        assert_eq!(object.faces().len(), 12);

        assert_eq!(object.named_groups().len(), 1);

        let cube = &object.named_groups()[0];
        assert_eq!(cube.name(), "Cube");
        assert_eq!(cube.face_start_index(), 0);
    }

    #[test]
    fn parse_two_triangles_without_object_names() {
        let plane = vec![
            "# o TriangleOne",
            "v -1.000000 0.000000 1.000000",
            "v 1.000000 0.000000 1.000000",
            "v -1.000000 0.000000 -1.000000",
            "vn -0.0000 1.0000 -0.0000",
            "vt 0.000000 0.000000",
            "vt 1.000000 0.000000",
            "vt 0.000000 1.000000",
            "s 0",
            "f 1/1/1 2/2/1 3/3/1",
            "# o TriangleTwo",
            "v 0.939709 -0.082245 0.425844",
            "v -1.060290 -0.082245 -1.574156",
            "v 0.939709 -0.082245 -1.574156",
            "vn -0.0000 1.0000 -0.0000",
            "vt 1.000000 0.000000",
            "vt 1.000000 1.000000",
            "vt 0.000000 1.000000",
            "s 0",
            "f 4/4/2 6/5/2 5/6/2",
        ];

        let object = Obj::parse(plane.into_iter().map(String::from), "Combined");

        assert_eq!(object.global_name(), "Combined");
        assert_eq!(object.vertices().len(), 6);
        assert_eq!(object.normals().len(), 2);
        assert_eq!(object.uv_coords().len(), 6);
        assert_eq!(object.faces().len(), 2);

        assert_eq!(object.named_groups().len(), 0);
    }
}
