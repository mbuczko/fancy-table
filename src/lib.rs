use charset::{Charset, TableChars};

mod ansi;
pub mod charset;
mod fancy;
mod juststr;

pub enum Layout {
    Slim,
    Fixed(usize),
    Expandable(usize),
}

pub enum Overflow {
    Wrap,
    Truncate,
}

pub enum Align {
    Center,
    Left,
    Right,
}

pub enum TitleAlign {
    LeftOffset(usize),
    RightOffset(usize),
}

pub enum Separator {
    Single,
    Double,
    Custom(char),
}

pub struct FancyTableOpts {
    pub title_align: TitleAlign,
    pub charset: Charset,
    pub headers_separator: Option<Separator>,
    pub rows_separator: Option<Separator>,
    pub max_lines: usize,
}

pub struct FancyTable<'a, T: AsRef<str>> {
    width: usize,
    chars: TableChars,
    padding: usize,
    columns: Vec<ColSpec>,
    headers: Vec<T>,
    rows_separator: Option<Separator>,
    headers_separator: Option<Separator>,
    title: Option<TitleSpec<'a>>,
}

pub struct FancyTableBuilder<'a, T: AsRef<str>> {
    padding: usize,
    max_lines: usize,
    rows_separator: Option<Separator>,
    headers_separator: Option<Separator>,
    charset: Charset,
    headers: Vec<T>,
    columns: Vec<ColSpec>,
    title: Option<&'a str>,
    title_align: TitleAlign,
}

struct ColSpec {
    width: usize,
    max_lines: usize,
    align: Align,
    layout: Layout,
    overflow: Overflow,
}

struct TitleSpec<'a> {
    title: &'a str,
    align: TitleAlign,
}

#[doc(hidden)]
pub use ansi::AnsiString;

#[doc(hidden)]
pub use juststr::JustedString;

#[doc(hidden)]
pub use juststr::Justify;
