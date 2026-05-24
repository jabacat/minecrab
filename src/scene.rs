pub trait Scene {
    fn enter(&self);
    fn update(&self);
    fn render(&self);
    fn exit(&self);
}

