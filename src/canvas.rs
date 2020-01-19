use crate::{DrawDestination, DrawError, DrawableUnit, Layer, UnitColor};
use data_structure::Pair;

const CANVAS_WIDTH: CanvasLattice = (40 - 2);
const CANVAS_HEIGHT: CanvasLattice = 30;

/// キャンバス内の描画先座標の成分となる型．
pub type CanvasLattice = usize;

/// キャンバス内の描画先座標を表す．
pub type CanvasItemPosition = Pair<CanvasLattice>;

/// キャンバス内の各点に保持される情報
#[derive(Debug, Clone, Copy)]
struct CanvasUnit<L> {
    drawable_unit: DrawableUnit,
    layer: L,
}

/// ストリームにゲーム情報を描画する．
pub struct Canvas<L> {
    /// 固定長の2次元キャンバスの各点の情報．
    lattices: [[Option<CanvasUnit<L>>; CANVAS_WIDTH]; CANVAS_HEIGHT],
}

impl<L> Canvas<L> {
    /// このキャンバスを描画した場合のサイズ (コンソール上の最小の正方形に対するサイズ)を返す．
    pub const fn size(&self) -> Pair<CanvasLattice> {
        Pair::new(CANVAS_WIDTH, CANVAS_HEIGHT)
    }

    /// 指定した点がこのキャンバスの領域内にあり，描画可能であるか返す．
    pub const fn is_drawable_at(&self, position: CanvasItemPosition) -> bool {
        let size = self.size();
        (position.x < size.x) & (position.y < size.y)
    }
}

impl<L: Layer> Canvas<L> {
    /// すべての点を空白にした状態のキャンバスを返す．
    pub fn empty_canvas() -> Self {
        Self {
            lattices: [[None; CANVAS_WIDTH]; CANVAS_HEIGHT],
        }
    }

    /// オブジェクトを指定した位置およびレイヤーに描画する．
    /// 指定した点に，より上位のレイヤーで描画されているオブジェクトが存在する場合，描画内容は更新されない．
    pub fn draw_unit(
        &mut self,
        drawable_unit: DrawableUnit,
        position: CanvasItemPosition,
        layer: L,
    ) {
        debug_assert!(position.x < CANVAS_WIDTH);
        debug_assert!(position.y < CANVAS_HEIGHT);
        let lattice = &mut self.lattices[position.y][position.x];
        match lattice {
            Some(l) if layer >= l.layer => {
                *lattice = Some(CanvasUnit {
                    drawable_unit,
                    layer,
                })
            }
            None => {
                *lattice = Some(CanvasUnit {
                    drawable_unit,
                    layer,
                })
            }
            _ => {}
        }
    }

    /// このキャンバスの内容をすべて文字列として書き込む．
    pub fn write_to<D: DrawDestination>(&self, destination: &mut D) -> Result<(), DrawError> {
        const CANVAS_BOUNDARY_COLOR: UnitColor = UnitColor::White;
        // top boundary
        for _ in 0..CANVAS_WIDTH + 2 {
            DrawableUnit::from_double_half_char('_', '_', CANVAS_BOUNDARY_COLOR)
                .write_to(destination)?;
        }
        destination.write_char('\n')?;
        //
        for lattice_row in self.lattices.iter() {
            // left boundary
            DrawableUnit::from_double_half_char(' ', '|', CANVAS_BOUNDARY_COLOR)
                .write_to(destination)?;
            // canvas contents
            for lattice in lattice_row.iter() {
                lattice
                    .map(|l| l.drawable_unit)
                    .unwrap_or(Self::empty_drawable_unit())
                    .write_to(destination)?;
            }
            // right boundary
            DrawableUnit::from_double_half_char('|', ' ', CANVAS_BOUNDARY_COLOR)
                .write_to(destination)?;
            //
            destination.write_char('\n')?;
        }
        // bottom boundary
        for _ in 0..CANVAS_WIDTH + 2 {
            DrawableUnit::from_single_full_char('￣', CANVAS_BOUNDARY_COLOR)
                .write_to(destination)?;
        }
        Ok(())
    }

    /// このキャンバス全体をクリアする．
    pub fn clear(&mut self) {
        for lattice_row in self.lattices.iter_mut() {
            for lattice in lattice_row.iter_mut() {
                *lattice = None;
            }
        }
    }

    fn empty_drawable_unit() -> DrawableUnit {
        DrawableUnit::from_double_half_char(' ', ' ', UnitColor::White)
    }
}
