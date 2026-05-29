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
white border. When highlighted, the background becomes a semi-opaque blue.

`Button` takes a special field `button` that implements the trait `ButtonType`.
This field dictates the return type of `check_mouse` via an associated type
stored within the trait.

#### The `ButtonType` trait

This trait acts as part of the model, view, and controller of the `Button`.

- `type T`: This is the associated type that dictates the generic type of
`GuiElement<T>` that the `Button` will implement. This type is the return type
of `act` and also the `Button`'s `check_mouse`. Be sure to set this in your
struct implementation.
- `fn get_text(&self) -> &str`: this function should return the text to render
on the button
- `fn act(&self, rl: &mut RaylibHandle) -> Option<Self::T>`: this function
will be called when the button is clicked. This part of the API will probably
have to be redesigned in the future.

This trait is probably one of the worst parts of the API design.
