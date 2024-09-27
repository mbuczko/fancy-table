use charset::{Charset, TableChars};

pub mod charset;
mod padstr;
mod fancy;

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

pub struct FancyTable<'a> {
    width: usize,
    chars: TableChars,
    padding: usize,
    headers_separator: Option<Separator>,
    headers: Vec<&'a str>,
    columns: Vec<ColSpec>,
    rows: Vec<Vec<&'a str>>,
    rows_separator: Option<Separator>,
    title: Option<TitleSpec<'a>>,
}

pub struct FancyTableBuilder<'a> {
    padding: usize,
    max_lines: usize,
    rows_separator: Option<Separator>,
    headers_separator: Option<Separator>,
    charset: Charset,
    headers: Vec<&'a str>,
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
