use raylib::prelude::*;
use std::ptr;

pub struct VecMesh {
    pub vao_id: u32,
    pub vbo_id: Vec<u32>,

    pub vertices: Vec<f32>,
    pub texcoords: Vec<f32>,
    pub texcoords2: Vec<f32>,
    pub normals: Vec<f32>,
    pub tangents: Vec<f32>,
    pub colors: Vec<u8>,
    pub indices: Vec<u16>,
}

impl VecMesh {
    pub fn new() -> VecMesh {
        return VecMesh {
            vao_id: 0,
            vbo_id: vec![],
            vertices: vec![],
            texcoords: vec![],
            texcoords2: vec![],
            normals: vec![],
            tangents: vec![],
            colors: vec![],
            indices: vec![],
        };
    }

    pub fn push_vertex(self: &mut Self, v: Vector3) {
        self.vertices.extend_from_slice(&[v.x, v.y, v.z]);
    }
    pub fn push_texcoord(self: &mut Self, tc: Vector2) {
        self.texcoords.extend_from_slice(&[tc.x, tc.y]);
    }
    pub fn push_normal(self: &mut Self, n: Vector3) {
        self.normals.extend_from_slice(&[n.x, n.y, n.z]);
    }
    pub fn push_tangent(self: &mut Self, t: Vector3) {
        self.tangents.extend_from_slice(&[t.x, t.y, t.z]);
    }
    pub fn push_color(self: &mut Self, c: Color) {
        self.colors.extend_from_slice(&[c.r, c.g, c.b]);
    }

    pub fn to_mesh(mut self: VecMesh) -> Mesh {
        let vertex_count = (self.vertices.len() / 3) as i32;
        let triangle_count =
            if self.indices.len() > 0 { self.indices.len() / 3 }
            else { self.vertices.len() / 9 }
            as i32;


        fn vec_to_ptr<T>(vec: Vec<T>) -> *mut T {
            if vec.len() == 0 {
                ptr::null_mut()
            } else {
                vec.leak().as_mut_ptr()
            }
        }

        let raw_mesh = ffi::Mesh {
            vaoId: self.vao_id,
            vboId: self.vbo_id.as_mut_ptr(),

            vertexCount: vertex_count,
            triangleCount: triangle_count,
            boneCount: 0,

            vertices: vec_to_ptr(self.vertices),
            texcoords: vec_to_ptr(self.texcoords),
            texcoords2: vec_to_ptr(self.texcoords2),
            normals: vec_to_ptr(self.normals),
            tangents: vec_to_ptr(self.tangents),
            colors: vec_to_ptr(self.colors),
            indices: vec_to_ptr(self.indices),
            boneIndices: ptr::null_mut(),
            boneWeights: ptr::null_mut(),
            animVertices: ptr::null_mut(),
            animNormals: ptr::null_mut(),
        };

        let mesh = unsafe { Mesh::from_raw(raw_mesh) };

        return mesh;
    }
}
