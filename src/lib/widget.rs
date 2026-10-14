use anyhow::Result;

pub trait WidgetTrait: Sized {
    fn new() -> Result<Self>;
    fn update(&mut self) -> Result<()>;
}
