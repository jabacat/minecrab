# The Graphical User Interface

As of when this document is current, the GUI system has emerged purely out of
scope creep from developing the pause menu. As such, the API is not well
-designed, since most of it has been written piecemeal until it was 
satisfactory for rendering the pause menu.

## Chronology (broadly speaking)

- Render a single rectangle using hardcoded coordinates
- Make the rectangle into a button with some more hardcoded control logic
- Extend the renderer to calculate button y-positions dynamically and render
two buttons in a column
- Render a submenu
- Extend the renderer further to be able to calculate button x-positions and
width dynamically and also render two columns of buttons
- Realize that I need to render a full-width back button after a two-column
layout
  - Option 1: Slightly extend the renderer again so that that was possible
  - Option 2: Significantly generalize the renderer
- Option 2 was chosen

## Current API

### The `GuiElement<T>` trait

All GUI elements must implement the GuiElement trait. This trait takes a very
object-oriented approach to describing the GUI: each element handles it's own
model, control (via `update`), and view (via `render`).

At the moment, we are not using `get_total_width` because the pause menu uses
a fixed total width equal to half the window width.

We are using `get_total_height` for layout purposes, i.e. centering the GUI.

`check_mouse` is called every frame to update any mouse inputs. If you are
constructing a new layout or container, you should probably propagate this and
call `check_mouse` on all of the elements in the container. `check_mouse` uses
the generic type in the trait to determine what to return. This behavior is
pretty hacky and was introduced in order to change the state of the pause menu
on button presses. In the pause menu, you'll find that everything returns a
`PauseMenuState`, and `set_state` is called on that return value. `check_mouse`
should likely be replaced by a more general `update` function that takes the
`rl: &mut RaylibHandle` in the future, but it suffices for now.

### The `ColLayout<T>` struct

`ColLayout` provides a basic column layout. The elements in `elements` will be
rendered vertically with a margin between each element given by `MARGIN_Y` in
`gui.rs`.

`ColLayout` will propagate `check_mouse` to its elements.

For convenience, a `col!` macro is provided, which constructs a `ColLayout`
with the necessary Boxes.

### The `RowLayout<T>` struct

`RowLayout` provides a basic row layout. The elements in `elements` will be
rendered horizontally with a mragin between each element given by `MARGIN_X`
in `gui.rs`.

`RowLayout` will propagate `check_mouse` to its elements.

`RowLayout`'s reported total height is the maximum computed height of one of
its elements.

For convenience, a `row!` macro is provided, which constructs a `RowLayout`
with the necessary Boxes.

### The `Button<BT: ButtonType>` struct

`Button` provides a basic button. `x` and `y` denote the position of the top
-left corner, and `width` and `height` denote their respective attributes.

`Button` renders a rectangular button with a semi-opaque black background and a
white border. When highlighted, the background becomes a semi-opaque blue. The
`text` is centered in both directions.

The `act` field takes a function pointer that serves as the controller of the
button, i.e. it processes inputs and changes state. Currently, act is only
called when the mouse is hovering over the button and the left mouse button is
pressed. `act` should take a `&mut RaylibHandle` and return an `Option<T>`.
The return type is a hacky workaround to allow buttons to mutate the state of
the pause menu by returning the desired state: the pause menu calls `set_state`
on the return value.
