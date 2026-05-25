# World and Chunk Design

## Struct Architecture

There is a single `World` object which controls all data about the world. The
`World` object has a list of `Chunk`s, each of which contains its own `Model` for
rendering.

(Perhaps in the future, the `Model` objects for each `Chunk` should be outsourced
to a more render-focused object, especially when the world and chunks store more
data about themselves than just an object for rendering.)

## World Interface

The public interface for a `World` specifies only a method to render (which, again,
should possibly in the future be outsourced to a render-specific object), and a
method to generate another chunk. The actual chunks making up a world are private.

## Chunk Interface

As far as access from a `World` object goes, the actual meshes in a `Chunk` are
private, so it has its own `render` method.

## Future Direction

We want the `World` and `Chunk` objects to be more focused on storing data rather
than render meshes and models, but so far there isn't a ton of data to access in
this manner.
