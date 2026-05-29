use raylib::prelude::*;

const MARGIN_X: i32 = 36;
const MARGIN_Y: i32 = 12;

const FONT_SIZE: i32 = 16;

const BUTTON_HEIGHT: i32 = 48;

const BUTTON_FG: Color = Color::WHITE;

const BUTTON_BG_INACTIVE: Color = Color::new(0, 0, 0, 240);
const BUTTON_BG_HOVER: Color = Color::new(64, 128, 192, 240);

const BUTTON_BORDER_COLOR: Color = Color::WHITE;

macro_rules! col {
    ( $e:expr ) => {
        Box::new(ColLayout::new(Box::new($e)))
    };
}

macro_rules! row {
    ( $e:expr ) => {
        Box::new(RowLayout::new(Box::new($e)))
    };
}

macro_rules! button {
    ( $t:expr, $a:expr ) => {
        Box::new(Button::new($t, $a))
    };
}

pub trait GuiElement<T> {
    // FIXME: we never use this method (width is calculated when needed, e.g. RowLayout::render)
    fn get_total_width(&self) -> i32;
    fn get_total_height(&self) -> i32;
    fn render(&mut self, d: &mut RaylibDrawHandle, x: i32, y: i32, width: i32);
    fn check_mouse(
        &mut self,
        rl: &mut RaylibHandle,
        mx: i32,
        my: i32,
        lmb_pressed: bool,
    ) -> Option<T>;
}

pub struct Button<T> {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    text: &'static str,
    act: Box<dyn Fn(&mut RaylibHandle) -> Option<T>>,

    hover: bool,
}

impl<T> Button<T> {
    pub fn new(text: &'static str, act: Box<dyn Fn(&mut RaylibHandle) -> Option<T>>) -> Self {
        Button {
            x: -1,
            y: -1,
            width: -1,
            height: BUTTON_HEIGHT,
            text,
            act,
            hover: false,
        }
    }
}

impl<T> GuiElement<T> for Button<T> {
    fn get_total_width(&self) -> i32 {
        todo!()
    }

    fn get_total_height(&self) -> i32 {
        // FIXME: maybe remove if we only use the constant BUTTON_HEIGHT?
        // although it's likely that this will be useful in the future
        // so actually probably DON't remove this property
        self.height
    }

    fn render(&mut self, d: &mut RaylibDrawHandle, x: i32, y: i32, width: i32) {
        // store current rendering params for mouse inputs
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = BUTTON_HEIGHT;

        d.draw_rectangle(
            x,
            y,
            width,
            BUTTON_HEIGHT,
            if self.hover {
                BUTTON_BG_HOVER
            } else {
                BUTTON_BG_INACTIVE
            },
        );
        d.draw_rectangle_lines(x, y, width, BUTTON_HEIGHT, BUTTON_BORDER_COLOR);

        let text_width = d.measure_text(self.text, FONT_SIZE);
        let text_x = (width - text_width) / 2 + x;
        let text_y = (BUTTON_HEIGHT - FONT_SIZE) / 2 + y;
        d.draw_text(self.text, text_x, text_y, FONT_SIZE, BUTTON_FG);
    }

    fn check_mouse(
        &mut self,
        rl: &mut RaylibHandle,
        mx: i32,
        my: i32,
        lmb_pressed: bool,
    ) -> Option<T> {
        self.hover = (self.x..(self.x + self.width)).contains(&(mx as i32))
            && (self.y..(self.y + self.height)).contains(&(my as i32));
        if self.hover && lmb_pressed {
            (self.act)(rl)
        } else {
            None
        }
    }
}

pub struct ColLayout<T> {
    elements: Box<[Box<dyn GuiElement<T>>]>,
}

impl<T> ColLayout<T> {
    pub fn new(elements: Box<[Box<dyn GuiElement<T>>]>) -> Self {
        Self { elements }
    }
}

impl<T> GuiElement<T> for ColLayout<T> {
    fn get_total_width(&self) -> i32 {
        todo!()
    }

    fn get_total_height(&self) -> i32 {
        self.elements
            .iter()
            .map(|e| e.get_total_height())
            .sum::<i32>()
            + (self.elements.len() as i32 - 1) * MARGIN_Y
    }

    fn render(&mut self, d: &mut RaylibDrawHandle, x: i32, y: i32, width: i32) {
        let mut current_y = y;

        // TODO: remove enumerate in refactor
        for (i, element) in self.elements.iter_mut().enumerate() {
            element.render(d, x, current_y, width);

            // bump current y position
            let element_height = element.get_total_height();
            current_y += element_height + MARGIN_Y;
        }
    }

    fn check_mouse(
        &mut self,
        rl: &mut RaylibHandle,
        mx: i32,
        my: i32,
        lmb_pressed: bool,
    ) -> Option<T> {
        // we don't need to do any mouse checks in this struct, but we need to propagate
        self.elements
            .iter_mut()
            .map(|e| e.check_mouse(rl, mx, my, lmb_pressed))
            .filter(|o| o.is_some())
            .last()
            .flatten()
    }
}

pub struct RowLayout<T> {
    elements: Box<[Box<dyn GuiElement<T>>]>,
}

impl<T> RowLayout<T> {
    pub fn new(elements: Box<[Box<dyn GuiElement<T>>]>) -> Self {
        Self { elements }
    }
}

impl<T> GuiElement<T> for RowLayout<T> {
    fn get_total_width(&self) -> i32 {
        todo!()
    }

    fn get_total_height(&self) -> i32 {
        self.elements
            .iter()
            .map(|e| e.get_total_height())
            .max()
            .unwrap_or(0)
    }

    fn render(&mut self, d: &mut RaylibDrawHandle, x: i32, y: i32, width: i32) {
        // XXX: we calculate element width here
        let num_elements = self.elements.len() as i32;
        let element_width = (width - MARGIN_X * (num_elements - 1)) / num_elements;

        for (i, element) in self.elements.iter_mut().enumerate() {
            let element_x = x + i as i32 * (element_width + MARGIN_X);

            element.render(d, element_x, y, element_width);
        }
    }

    fn check_mouse(
        &mut self,
        rl: &mut RaylibHandle,
        mx: i32,
        my: i32,
        lmb_pressed: bool,
    ) -> Option<T> {
        // we don't need to do any mouse checks in this struct, but we need to propagate
        self.elements
            .iter_mut()
            .map(|e| e.check_mouse(rl, mx, my, lmb_pressed))
            .filter(|o| o.is_some())
            .last()
            .flatten()
    }
}
