use std::{error::Error, path::Path};

use crate::{Color, CompositeShape};

pub trait Render {
    type Error: Error;

    fn init(&mut self, _background_color: Color) -> Result<(), Self::Error> {
        Ok(())
    }

    fn load_font(&mut self, name: impl AsRef<str>, path: impl AsRef<Path>) -> Result<(), Self::Error>;

    #[allow(unused_variables)]
    fn set_dimensions(&mut self, physical_width: u32, physical_height: u32, device_pixel_ratio: f64) {}

    fn render(&mut self, node: &mut dyn CompositeShape) -> Result<bool, Self::Error>;
}
