extern crate console;
extern crate itertools;

use crate::{DrawableUnit, Layer, UiCanvas, UiLattice, UiPosition, UnitColor};
use geometry::Rectangle;
use itertools::Itertools;
use std::collections::VecDeque;

/// UIに表示する1行のメッセージを表す．
#[derive(Debug, Clone)]
struct MessageLine {
    /// メッセージの内容．
    units: VecDeque<DrawableUnit>,
    /// このメッセージの末尾に新たなメッセージを追加可能か．
    is_growable: bool,
    /// このメッセージが保持する最大文字数．
    max_message_length: usize,
}

/// UIに表示するメッセージ内容を管理する．
#[derive(Debug, Clone)]
pub struct MessageBuffer {
    /// メッセージ行
    lines: VecDeque<MessageLine>,
    /// メッセージを保持する最大行数．
    max_line_count: usize,
    /// 各行のメッセージが保持する最大文字数．
    max_message_length: usize,
    /// メッセージ欄の境界線に描画するオブジェクト
    border_unit: DrawableUnit,
}

impl MessageLine {
    /// 拡張可能な空のメッセージ行を返す．
    /// # Params
    /// 1. max_message_length メッセージの各行が保持する最大文字数．
    /// これよりも多くの文字数が追加された場合，古い文字から削除される．
    fn empty_growable_line(max_message_length: usize) -> Self {
        Self {
            units: VecDeque::with_capacity(max_message_length),
            is_growable: true,
            max_message_length,
        }
    }

    /// 空の拡張不可能なメッセージ行を返す．
    fn empty_ungrowable_line() -> Self {
        Self {
            units: VecDeque::new(),
            is_growable: false,
            max_message_length: 0,
        }
    }

    /// このメッセージに含まれている文字を先頭から順に列挙する．
    fn units(&self) -> impl Iterator<Item = &DrawableUnit> {
        self.units.iter()
    }

    /// このメッセージに含まれている文字数を返す．
    fn len(&self) -> usize {
        self.units.len()
    }

    /// このメッセージに新たなメッセージを追加する．
    /// # Panics
    /// このメッセージが拡張不可能な場合．
    fn append_message(&mut self, units: &[DrawableUnit]) {
        assert!(self.is_growable);
        for &unit in units {
            if self.units.len() == self.max_message_length {
                self.units.pop_front().unwrap();
            }
            self.units.push_back(unit);
        }
    }

    /// このメッセージを拡張不可能な状態にする．
    /// # Panics
    /// このメッセージがすでに拡張不可能な場合．
    fn disable_growable(&mut self) {
        assert!(self.is_growable);
        self.is_growable = false;
    }
}

impl MessageBuffer {
    /// メッセージ内容を管理するオブジェクトを初期化する．
    /// # Params
    /// 1. `max_line_count` このバッファが保持する最大メッセージ行数．
    /// これよりも多くの行数が追加された場合，古いメッセージから削除される．
    /// 1. `max_message_length` メッセージの各行が保持する最大文字数．
    /// これよりも多くの文字数が追加された場合，古い文字から削除される．
    /// 1. `border_unit` メッセージ欄の境界線に描画されるオブジェクト．
    pub fn new(
        max_line_count: usize,
        max_message_length: usize,
        border_unit: DrawableUnit,
    ) -> Self {
        Self {
            lines: VecDeque::new(),
            max_line_count,
            max_message_length,
            border_unit,
        }
    }

    /// このバッファに制御文字なしのメッセージを追加する．
    /// # Params
    /// 1. text 追加する文字．
    ///
    /// # Panics
    /// 指定された文字列に制御文字が含まれている場合．
    pub fn add_text<T: AsRef<str>>(&mut self, text: T, color: UnitColor) {
        // 現在保持している最終行が拡張可能な場合は，そこにメッセージを追加する．
        // その他の場合は，新しいメッセージ行を作成する．このとき，すでにメッセージ行数が上限に達している場合は，もっとも古い行が削除される．
        match self.lines.back_mut() {
            Some(last_line) => {
                if last_line.is_growable {
                    last_line
                        .append_message(&DrawableUnit::create_units_from(text.as_ref(), color));
                } else {
                    let mut line = MessageLine::empty_growable_line(self.max_message_length);
                    line.append_message(&DrawableUnit::create_units_from(text.as_ref(), color));
                    self.add_new_message_line(line);
                }
            }
            None => {
                let mut line = MessageLine::empty_growable_line(self.max_message_length);
                line.append_message(&DrawableUnit::create_units_from(text.as_ref(), color));
                self.add_new_message_line(line);
            }
        }
    }

    // このバッファに改行を加える．
    pub fn add_newline(&mut self) {
        // 現在保持している最終行が拡張可能な場合は，そのメッセージを拡張不可にする
        // その他の場合は，新しい空のメッセージ行を作成する．このとき，すでにメッセージ行数が上限に達している場合は，もっとも古い行が削除される．
        match self.lines.back_mut() {
            Some(last_line) => {
                if last_line.is_growable {
                    last_line.disable_growable();
                } else {
                    self.add_new_message_line(MessageLine::empty_ungrowable_line());
                }
            }
            None => self.add_new_message_line(MessageLine::empty_ungrowable_line()),
        }
    }

    /// キャンバス上にメッセージ内容を描画する．
    /// # Params
    /// 1. ui_canvas 描画対象となるキャンバス．
    /// 1. region_on_canvas メッセージ内容の描画領域．
    ///
    /// # Panics
    /// 描画領域の幅が0である場合．
    pub fn draw_message<L: Layer>(
        &self,
        ui_canvas: &mut UiCanvas<'_, L>,
        region_on_canvas: Rectangle<UiLattice>,
        layer: L,
    ) {
        // 枠で囲み中を空白でクリア
        self.draw_border_and_fill_inner(ui_canvas, region_on_canvas, layer);
        // 枠内部の描画領域をとる
        let region_on_canvas = {
            let top = region_on_canvas.top() + 1;
            let bottom = region_on_canvas.bottom() - 1;
            let left = region_on_canvas.left() + 1;
            let right = region_on_canvas.right() - 1;
            Rectangle::from_corners(UiPosition::new(left, top), UiPosition::new(right, bottom))
        };
        // 次のメッセージの末尾部分を何行目に描画するか
        let mut current_end_row = region_on_canvas.bottom();
        // 表示すべきメッセージを最後の行から処理していく
        for line in self.lines.iter().rev() {
            use std::cmp::max;
            // このメッセージの表示に何行消費するか計算
            let required_line_count = max(div_ceil(line.len(), region_on_canvas.width()), 1);
            // 何行目からこのメッセージの描画を始めるか．これは負になることもあるのでキャストが必須
            let start_row = current_end_row as isize - required_line_count as isize + 1;
            // もし，メッセージ表示行が描画領域内に入っていなければ，その時点で描画を打ち切り
            {
                let end_row = start_row + required_line_count as isize - 1;
                if end_row < region_on_canvas.top() as isize {
                    break;
                }
            }
            for (row_on_canvas, units) in line
                .units()
                .chunks(region_on_canvas.width())
                .into_iter()
                .enumerate()
                .map(|(index, units)| (start_row + index as isize, units))
                .filter(|(row_on_canvas, _units)| *row_on_canvas >= region_on_canvas.top() as isize)
                .map(|(row_on_canvas, units)| (row_on_canvas as usize, units))
            {
                for (column_on_canvas, &unit) in units
                    .into_iter()
                    .enumerate()
                    .map(|(index, unit)| (index + region_on_canvas.left(), unit))
                {
                    let position = UiPosition::new(column_on_canvas, row_on_canvas);
                    ui_canvas.draw_unit(unit, position, layer);
                }
            }
            // 次のメッセージは，今描画した行よりも上に表示する．
            // このとき，次に描画するメッセージの終了行が負になる場合は，そのメッセージは決して描画領域内には入らないので，その時点で処理を打ち切る．
            if (current_end_row as isize - required_line_count as isize) < 0 {
                break;
            } else {
                current_end_row -= required_line_count;
            }
        }
    }

    // メッセージ描画領域を囲む厚さ1の枠を描画し，さらにその内部を空白で埋める．
    fn draw_border_and_fill_inner<L: Layer>(
        &self,
        ui_canvas: &mut UiCanvas<'_, L>,
        region_on_canvas: Rectangle<UiLattice>,
        layer: L,
    ) {
        let top = region_on_canvas.top();
        let bottom = region_on_canvas.bottom();
        let left = region_on_canvas.left();
        let right = region_on_canvas.right();
        for row in top..bottom + 1 {
            let is_horizontal_border = row == top || row == bottom;
            for column in left..right + 1 {
                let is_vertical_border = column == left || column == right;
                let is_border = is_horizontal_border || is_vertical_border;
                // 描画領域の境界上は枠，その他は空白
                let unit = if is_border {
                    self.border_unit
                } else {
                    DrawableUnit::from_double_half_char(' ', ' ', UnitColor::White)
                };
                let position = UiPosition::new(column, row);
                ui_canvas.draw_unit(unit, position, layer);
            }
        }
    }

    /// このメッセージバッファに新しいメッセージ行を追加する．
    /// このとき，すでにメッセージ行数が上限に達している場合は，もっとも古い行が削除される．
    fn add_new_message_line(&mut self, new_message_line: MessageLine) {
        if self.lines.len() == self.max_line_count {
            self.lines.pop_front();
        }
        self.lines.push_back(new_message_line);
    }
}

/// `x / y`を下回らない最小の整数を返す．
/// # Panics
/// if `y` == 0
/// # Examples
/// ```rust,ignore
/// use cui_gaming::message_buffer::div_ceil;
///
/// assert_eq!(3, div_ceil(6, 2));
/// assert_eq!(3, div_ceil(5, 2));
/// assert_eq!(2, div_ceil(4, 2));
///
/// // Panics if y is 0
/// //div_ceil(1, 0));
/// ```
fn div_ceil(x: usize, y: usize) -> usize {
    x / y + if x % y == 0 { 0 } else { 1 }
}

#[cfg(test)]
mod message_line_tests {
    use super::*;
    #[test]
    fn test_append_message() {
        let units = [
            DrawableUnit::from_double_half_char(' ', ' ', UnitColor::White),
            DrawableUnit::from_double_half_char(' ', ' ', UnitColor::Red),
            DrawableUnit::from_double_half_char(' ', ' ', UnitColor::Yellow),
        ];
        let mut message_line = MessageLine::empty_growable_line(5);
        message_line.append_message(&units);
        {
            let current_units = message_line.units().map(|u| *u).collect::<Vec<_>>();
            assert_eq!(&units, current_units.as_slice());
        }
        message_line.append_message(&units);
        {
            let current_units = message_line.units().map(|u| *u).collect::<Vec<_>>();
            assert_eq!(units[1], current_units[0]);
            assert_eq!(units[2], current_units[1]);
            assert_eq!(units[0], current_units[2]);
            assert_eq!(units[1], current_units[3]);
            assert_eq!(units[2], current_units[4]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::div_ceil;
    #[test]
    fn test_div_ceil() {
        assert_eq!(3, div_ceil(6, 2));
        assert_eq!(3, div_ceil(5, 2));
        assert_eq!(2, div_ceil(4, 2));
        //
        assert_eq!(4, div_ceil(4, 1));
        assert_eq!(1, div_ceil(1, 1));
    }
    #[should_panic]
    #[test]
    fn test_div_ceil_panic() {
        div_ceil(1, 0);
    }
    #[should_panic]
    #[test]
    fn test_div_ceil_panic2() {
        div_ceil(0, 0);
    }
}
