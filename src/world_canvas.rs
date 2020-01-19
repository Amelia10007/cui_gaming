use crate::{Canvas, CanvasItemPosition, CanvasLattice, DrawableUnit, Layer};
use data_structure::Pair;

pub type WorldLattice = isize;
pub type WorldPosition = Pair<WorldLattice>;

/// ゲームフィールド内の情報を描画する際の座標変換を行う．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Reference {
    /// キャンバス内の参照点．ゲームフィールド内参照点は，この点に描画される．
    reference_canvas_position: CanvasItemPosition,
    /// ゲームフィールド内の参照点．この点がキャンバス内参照点に描画される．
    reference_world_position: WorldPosition,
}

/// ゲームフィールド内の情報を描画する．
pub struct WorldCanvas<'a, L: Layer> {
    /// 描画先キャンバス．
    canvas: &'a mut Canvas<L>,
    /// 座標変換
    reference: Reference,
}

impl Reference {
    pub fn new(
        reference_canvas_position: CanvasItemPosition,
        reference_world_position: WorldPosition,
    ) -> Self {
        Self {
            reference_canvas_position,
            reference_world_position,
        }
    }

    /// フィールド上の指定した点を描画する場合の，キャンバス上の描画先となる点を返す．
    /// # Returns
    /// キャンバス上の描画点`p`を`Some(p)`として返す．
    ///
    /// フィールド上の点がキャンバス外の点に対応する場合は，`None`を返す．
    fn canvas_position_of<L>(
        &self,
        world_position: WorldPosition,
        canvas: &Canvas<L>,
    ) -> Option<CanvasItemPosition> {
        let canvas_position = {
            // キャンバス上の点の算出に失敗 (オーバーフロー)した場合は，指定された点は描画できないので無効値を返す．
            let offset = world_position - self.reference_world_position;
            (offset + self.reference_canvas_position.try_cast().ok()?)
                .try_cast::<CanvasLattice>()
                .ok()?
        };
        // 算出点がキャンバス内に存在するかチェック
        if canvas.is_drawable_at(canvas_position) {
            Some(canvas_position)
        } else {
            None
        }
    }
}

impl<'a, L: Layer> WorldCanvas<'a, L> {
    /// 描画先となるキャンバスを指定して，ゲームフィールド描画用のキャンバスを生成する．
    pub fn from_canvas(canvas: &'a mut Canvas<L>, reference: Reference) -> Self {
        Self { canvas, reference }
    }

    /// フィールド上のオブジェクトを描画する．
    pub fn draw_unit(
        &mut self,
        drawable_unit: DrawableUnit,
        world_position: WorldPosition,
        layer: L,
    ) {
        if let Some(canvas_position) = self
            .reference
            .canvas_position_of(world_position, self.canvas)
        {
            self.canvas.draw_unit(drawable_unit, canvas_position, layer)
        }
    }
}

#[cfg(test)]
mod reference_tests {
    use super::*;
    #[test]
    fn test_canvas_position_of() {
        let canvas = Canvas::<i32>::empty_canvas();
        let reference_canvas_position = CanvasItemPosition::new(10, 20);
        let reference_world_position = WorldPosition::new(-1, 5);
        let reference = Reference::new(reference_canvas_position, reference_world_position);
        assert_eq!(
            Some(reference_canvas_position),
            reference.canvas_position_of(reference_world_position, &canvas)
        );
        assert_eq!(
            Some(reference_canvas_position + CanvasItemPosition::new(10, 1)),
            reference.canvas_position_of(
                reference_world_position + WorldPosition::new(10, 1),
                &canvas
            )
        );
        // フィールドの点がキャンバス範囲外の点に対応する場合はNoneが返る
        assert_eq!(
            None,
            reference.canvas_position_of(
                reference_world_position - canvas.size().try_cast::<WorldLattice>().unwrap(),
                &canvas
            )
        );
    }
}
