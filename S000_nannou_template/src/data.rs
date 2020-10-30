use crate::graphics::{Normal, Vertex};

pub const NUM_VERTICES: usize = 4;
pub const NUM_NORMALS: usize = 4;

pub const VERTICES: [Vertex; NUM_VERTICES] = [
    Vertex {
        position: (-1.0, -1.0, 0.0),
    },
    Vertex {
        position: (1.0, -1.0, 0.0),
    },
    Vertex {
        position: (-1.0, 1.0, 0.0),
    },
    Vertex {
        position: (1.0, 1.0, 0.0),
    },
];

pub const NORMALS: [Normal; NUM_NORMALS] = [
    Normal {
        normal: (0.0, 0.0, 1.0),
    },
    Normal {
        normal: (0.0, 0.0, 1.0),
    },
    Normal {
        normal: (0.0, 0.0, 1.0),
    },
    Normal {
        normal: (0.0, 0.0, 1.0),
    },
];

pub const INDICES: [u16; 6] = [0, 1, 2, 2, 1, 3];
