# Renderer Design

## WorldRenderer

A `WorldRenderer` keeps a list of chunk meshes to render. These are produced upon
creation of a new chunk, and the `render` method simply renders all of them. This
way, there is separation between world/chunk data and the meshes used to render
them.

## Future Direction

The chunk meshes will in the future need to be indexed by chunk coordinates, for
when chunks can be modified after creation.
