// All block types we will have.
#[derive(Clone, Copy, PartialEq)]
pub enum BlockData {
    AIR,
    GRASS,
    DIRT,
    STONE,
    WOOD,
    LEAVES,
}

// Each type of block has six faces which can be rendered. Each face (until we
// add non-cubic blocks) corresponds to a single square in the texture atlas, so
// this is a store for which faces exist to be rendered.
pub struct BlockTextureCoordinates {
    pub right: (f32, f32),
    pub left: (f32, f32),
    pub top: (f32, f32),
    pub bottom: (f32, f32),
    pub front: (f32, f32),
    pub back: (f32, f32),
}

// TODO: stash these in a file somewhere?

pub const GRASS_COORDS: BlockTextureCoordinates = BlockTextureCoordinates {
    right: (0.1, 0.0),
    left: (0.1, 0.0),
    top: (0.0, 0.0),
    bottom: (0.2, 0.0),
    front: (0.1, 0.0),
    back: (0.1, 0.0),
};
