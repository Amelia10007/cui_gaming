extern crate console;
extern crate unicode_width;

use std::fmt;

pub type UnitColor = console::Color;

/// 描画先となれる型であることを表す．
pub trait DrawDestination: fmt::Write {}

/// 描画時のエラーを表す型．
pub type DrawError = fmt::Error;

/// 描画する内容の最小単位を表す．
/// このオブジェクトは，コンソール上の最小の正方形領域内に描画されることが保証されている．
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DrawableUnit {
    left: char,
    right: Option<char>,
    color: UnitColor,
}

// Auto trait implementation
impl<W: fmt::Write> DrawDestination for W {}

impl DrawableUnit {
    /// 描画時の占有領域がコンソール上の最小の正方形となるような描画単位を返す．
    /// # Panics on Debug Build
    /// Releaseビルド時にはチェックは行われない．
    /// 1. 描画時の占有領域が正方形とならない場合．
    pub fn from_single_full_char(c: char, color: UnitColor) -> Self {
        debug_assert_eq!(Some(2), unicode_width::UnicodeWidthChar::width(c));
        Self {
            left: c,
            right: None,
            color,
        }
    }

    /// 描画時の占有領域がコンソール上の最小の正方形となるような描画単位を返す．
    /// # Panics on Debug Build
    /// Releaseビルド時にはチェックは行われない．
    /// 1. 描画時の占有領域が正方形とならない場合．
    pub fn from_double_half_char(left: char, right: char, color: UnitColor) -> Self {
        debug_assert_eq!(Some(1), unicode_width::UnicodeWidthChar::width(left));
        debug_assert_eq!(Some(1), unicode_width::UnicodeWidthChar::width(right));
        Self {
            left,
            right: Some(right),
            color,
        }
    }

    /// 指定した文字列から，正方形領域内に描画可能な単位の繰り返しを生成して返す．
    /// # Panics on Debug Build
    /// コンソールへの描画時に幅が1か2以外の文字が含まれる場合
    pub fn create_units_from(s: &str, color: UnitColor) -> Vec<Self> {
        // すべての文字は幅1か2でなければならない
        debug_assert!(s
            .chars()
            .all(|c| match unicode_width::UnicodeWidthChar::width(c) {
                Some(w) if w == 1 || w == 2 => true,
                _ => false,
            }));
        let mut units = vec![];
        let mut previous = None;
        for (c, width) in s.chars().map(|c| {
            (
                c,
                unicode_width::UnicodeWidthChar::width(c)
                    .expect("Char for drawable unit must have width on console."),
            )
        }) {
            if width == 1 {
                match previous {
                    Some(p) => {
                        units.push(Self::from_double_half_char(p, c, color));
                        previous = None;
                    }
                    None => previous = Some(c),
                }
            } else if width == 2 {
                match previous {
                    Some(p) => {
                        units.push(Self::from_double_half_char(p, ' ', color));
                        units.push(Self::from_single_full_char(c, color));
                        previous = None;
                    }
                    None => units.push(Self::from_single_full_char(c, color)),
                }
            }
        }
        // 最後に，まだ追加していない半角文字があれば追加 (全角文字がここまで残っていることはありえない)
        if let Some(c) = previous {
            units.push(Self::from_double_half_char(c, ' ', color));
        }
        units
    }

    /// このオブジェクトを指定した描画先に書き込む．
    /// このオブジェクトは，コンソール上の最小の正方形領域内に描画されることが保証されている．
    pub fn write_to<D: DrawDestination>(&self, destination: &mut D) -> Result<(), DrawError> {
        use std::iter::FromIterator;
        let colored_str = {
            let s = match self.right {
                Some(right) => String::from_iter(&[self.left, right]),
                None => self.left.to_string(),
            };
            let temp_style = console::style(s);
            match self.color {
                UnitColor::Black => temp_style.black(),
                UnitColor::Blue => temp_style.blue(),
                UnitColor::Cyan => temp_style.cyan(),
                UnitColor::Green => temp_style.green(),
                UnitColor::Magenta => temp_style.magenta(),
                UnitColor::Red => temp_style.red(),
                UnitColor::White => temp_style.white(),
                UnitColor::Yellow => temp_style.yellow(),
            }
        };
        destination.write_fmt(format_args!("{}", colored_str))?;
        Ok(())
    }
}

#[cfg(test)]
mod test_util {
    use super::*;
    pub fn get_string_without_style(units: &[DrawableUnit]) -> String {
        let mut s = String::new();
        for unit in units {
            s.push(unit.left);
            if let Some(right) = unit.right {
                s.push(right);
            }
        }
        s
    }
}

#[cfg(test)]
mod tests_from_single_char {
    use super::*;
    use super::test_util::get_string_without_style;
    #[test]
    fn pass() {
        let unit = DrawableUnit::from_single_full_char('あ', UnitColor::White);
        assert_eq!("あ", get_string_without_style(&[unit]));
    }
    #[should_panic]
    #[test]
    fn panic_by_half_char_ascii() {
        DrawableUnit::from_single_full_char('a', UnitColor::White);
    }
    #[should_panic]
    #[test]
    fn panic_by_half_char_non_ascii() {
        DrawableUnit::from_single_full_char('◎', UnitColor::White);
    }
    #[should_panic]
    #[test]
    fn panic_by_control_char() {
        DrawableUnit::from_single_full_char('\n', UnitColor::White);
    }
}

#[cfg(test)]
mod tests_from_double_half_char {
    use super::*;
    use super::test_util::get_string_without_style;
    #[test]
    fn pass_only_ascii() {
        let unit = DrawableUnit::from_double_half_char('a', 'b', UnitColor::White);
        assert_eq!("ab", get_string_without_style(&[unit]));
    }
    #[test]
    fn pass_only_non_ascii() {
        let unit = DrawableUnit::from_double_half_char('●', '◎', UnitColor::White);
        assert_eq!("●◎", get_string_without_style(&[unit]));
    }
    #[test]
    fn pass_ascii_and_non_ascii() {
        let unit = DrawableUnit::from_double_half_char('a', '◎', UnitColor::White);
        assert_eq!("a◎", get_string_without_style(&[unit]));
    }
    #[should_panic]
    #[test]
    fn panic_by_including_full_char() {
        DrawableUnit::from_double_half_char('a', 'あ', UnitColor::White);
    }
    #[should_panic]
    #[test]
    fn panic_by_control_char() {
        DrawableUnit::from_double_half_char('a', '\n', UnitColor::White);
    }
}

#[cfg(test)]
mod tests_create_units_from {
    use super::*;
    use super::test_util::get_string_without_style;
    #[test]
    fn only_ascii() {
        let units = DrawableUnit::create_units_from("abcdef", UnitColor::White);
        assert_eq!("abcdef", get_string_without_style(&units));
    }
    #[test]
    fn only_ascii_with_last_whitespace() {
        let units = DrawableUnit::create_units_from("abcdefg", UnitColor::White);
        assert_eq!("abcdefg ", get_string_without_style(&units));
    }
    #[test]
    fn only_full_char() {
        let units = DrawableUnit::create_units_from("あいうえお", UnitColor::White);
        assert_eq!("あいうえお", get_string_without_style(&units));
    }
    #[test]
    fn combining_half_and_full_char() {
        let units = DrawableUnit::create_units_from("あaいiuうe", UnitColor::White);
        assert_eq!("あa いiuうe ", get_string_without_style(&units));
    }
    #[should_panic]
    #[test]
    fn panic_by_control_char() {
        let source = "abc\td";
        DrawableUnit::create_units_from(&source, UnitColor::White);
    }
}
