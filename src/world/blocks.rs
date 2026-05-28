// All block types we will have.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BlockData {
    AIR,
    GRASS,
    DIRT,
    STONE,
    WOOD,
    LEAVES,
    BEDROCK,
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

// A block whose faces all have an identical texture.
macro_rules! monotextured_block {
    ($x:expr, $y:expr) => {BlockTextureCoordinates {
        left: ($x, $y),
        right: ($x, $y),
        top: ($x, $y),
        bottom: ($x, $y),
        front: ($x, $y),
        back: ($x, $y),
    }}
}

// A block with a unique top texture, bottom texture, and side texture.
// For example, grass and wood both do this.
macro_rules! cylindrical_textured_block {
    ($top:expr, $bottom:expr, $side:expr) => {BlockTextureCoordinates {
        left: $side,
        right: $side,
        top: $top,
        bottom: $bottom,
        front: $side,
        back: $side,
    }}
}

// If a block is invalid, we render with the Adam pickaxe texture
const TEX_COORDS_ERROR: BlockTextureCoordinates = monotextured_block!(0.8, 0.0);

// For completeness, air will give us the Jake texture in case it gets rendered
// by accident, so we know what is happening
const TEX_COORDS_AIR: BlockTextureCoordinates = monotextured_block!(0.9, 0.0);

const TEX_COORDS_GRASS: BlockTextureCoordinates = cylindrical_textured_block!(
    (0.0, 0.0), (0.2, 0.0), (0.1, 0.0)
);

const TEX_COORDS_DIRT: BlockTextureCoordinates = monotextured_block!(0.2, 0.0);

const TEX_COORDS_STONE: BlockTextureCoordinates = monotextured_block!(0.3, 0.0);

const TEX_COORDS_WOOD: BlockTextureCoordinates = cylindrical_textured_block!(
    (0.4, 0.1), (0.4, 0.1), (0.3, 0.1)
);

const TEX_COORDS_LEAVES: BlockTextureCoordinates = monotextured_block!(0.5, 0.1);

impl BlockTextureCoordinates {

    pub fn new(block_type: BlockData) -> BlockTextureCoordinates {
        match block_type {
            BlockData::AIR => TEX_COORDS_AIR,
            BlockData::GRASS => TEX_COORDS_GRASS,
            BlockData::DIRT => TEX_COORDS_DIRT,
            BlockData::STONE => TEX_COORDS_STONE,
            BlockData::WOOD => TEX_COORDS_WOOD,
            BlockData::LEAVES => TEX_COORDS_LEAVES,
            // Not implemented yet!
            _ => TEX_COORDS_ERROR,
        }
    }
}
