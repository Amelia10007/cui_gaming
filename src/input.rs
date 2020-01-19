extern crate console;

use console::{Key, Term};
use std::error::Error;
use std::str::FromStr;

/// ユーザーからのキー入力を管理する．
#[derive(Debug)]
pub struct KeyboardInput {
    terminal: console::Term,
}

impl KeyboardInput {
    /// 新しくスレッドを生成し，キー入力監視を開始する．
    pub fn new() -> Self {
        Self {
            terminal: Term::buffered_stdout(),
        }
    }

    /// キー入力が発生するまで待機し，入力されたキーを返す．
    pub fn read_key(&self) -> std::io::Result<Key> {
        self.terminal.read_key()
    }

    /// 1行文字列が入力されるまで待機し，その文字列を返す．
    pub fn read_line(&self) -> std::io::Result<String> {
        self.terminal.read_line()
    }

    /// 1行文字列が入力されるまで待機し，その文字列をパースして返す．
    pub fn parse_line<T>(&self) -> Result<T, Box<dyn Error + 'static>>
    where
        T: FromStr,
        T::Err: Error + 'static,
    {
        self.read_line()
            .map_err(|e| Box::new(e) as Box<_>)
            .and_then(|line| T::from_str(&line).map_err(|e| Box::new(e) as Box<_>))
    }
}
