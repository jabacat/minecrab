# World and Chunk Design

## Struct Architecture

There is a single `World` object which controls all data about the world. The
`World` object has a list of `Chunk`s, each of which contains its own voxel data.

## World Interface

The world also has methods to read and set individual chunk and block data, as
well as to convert between global and chunk-internal coordinates.

## Rendering

Rendering is outsourced to a `WorldRenderer` object.
