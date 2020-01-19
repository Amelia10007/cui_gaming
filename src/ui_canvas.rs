use crate::{Canvas, Layer};
use crate::DrawableUnit;
use data_structure::Pair;

pub type UiLattice = usize;
pub type UiPosition = Pair<UiLattice>;

pub struct UiCanvas<'a, L> {
    canvas: &'a mut Canvas<L>,
}

impl<'a, L: Layer> UiCanvas<'a, L> {
    pub fn from_canvas(canvas: &'a mut Canvas<L>) -> Self {
        Self { canvas }
    }

    pub fn draw_unit(&mut self, drawable_unit: DrawableUnit, ui_position: UiPosition, layer: L) {
        let canvas_position = ui_position.into();
        self.canvas.draw_unit(drawable_unit, canvas_position, layer)
    }
}
