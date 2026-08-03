use std::cmp::min;

use crate::{
    Align, ColSpec, FancyTable, FancyTableBuilder, FancyTableOpts, Layout, Overflow, Separator,
    TitleAlign, TitleSpec, Width,
    ansi::{self, Overflow as AnsiOverflow},
    charset::Charset,
    juststr::{JustedString, Justify},
};

const DEFAULT_COLUMN_WIDTH: usize = 10;

impl Default for FancyTableOpts {
    fn default() -> Self {
        Self {
            title_align: TitleAlign::LeftOffset(4),
            charset: Charset::Modern,
            headers_separator: Some(Separator::Double),
            rows_separator: None,
            max_lines: 3,
        }
    }
}

impl<'a, T: AsRef<str>> FancyTableBuilder<'a, T> {
    fn new(opts: FancyTableOpts) -> Self {
        Self {
            headers: Vec::new(),
            columns: Vec::new(),
            padding: 1,
            width: Width::Fixed(80),
            charset: opts.charset,
            rows_separator: opts.rows_separator,
            headers_separator: opts.headers_separator,
            max_lines: opts.max_lines,
            title: None,
            title_align: opts.title_align,
        }
    }
    fn add_column_spec(
        mut self,
        width: usize,
        max_lines: usize,
        layout: Layout,
        align: Align,
        overflow: Overflow,
    ) -> Self {
        self.columns.push(ColSpec {
            width: width.max(1),
            layout,
            align,
            overflow,
            max_lines,
        });
        self
    }

    pub fn add_column(
        mut self,
        header: Option<T>,
        layout: Layout,
        align: Align,
        overflow: Overflow,
        max_lines: usize,
    ) -> Self {
        let len = match layout {
            Layout::Fixed(f) => f,
            _ => header
                .as_ref()
                .map(|h| h.as_ref().chars().count())
                .unwrap_or(DEFAULT_COLUMN_WIDTH),
        };
        if let Some(header) = header {
            self.headers.push(header);
        }
        self.add_column_spec(len, max_lines, layout, align, overflow)
    }
    pub fn add_column_named(self, header: T, layout: Layout) -> Self {
        self.add_column_named_with_align(header, layout, Align::Left)
    }
    pub fn add_column_named_wrapping(self, header: T, layout: Layout) -> Self {
        self.add_column_named_wrapping_with_align(header, layout, Align::Left)
    }
    pub fn add_column_named_with_align(mut self, header: T, layout: Layout, align: Align) -> Self {
        let len = header.as_ref().len();
        let max_lines = self.max_lines;

        self.headers.push(header);
        self.add_column_spec(len, max_lines, layout, align, Overflow::Truncate)
    }
    pub fn add_column_named_wrapping_with_align(
        mut self,
        header: T,
        layout: Layout,
        align: Align,
    ) -> Self {
        let len = header.as_ref().len();
        let max_lines = self.max_lines;

        self.headers.push(header);
        self.add_column_spec(len, max_lines, layout, align, Overflow::Wrap)
    }
    pub fn add_title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
    pub fn add_title_with_align(mut self, title: &'a str, align: TitleAlign) -> Self {
        self.title_align = align;
        self.add_title(title)
    }
    pub fn padding(mut self, padding: usize) -> Self {
        self.padding = padding;
        self
    }
    pub fn hseparator(mut self, separator: Option<Separator>) -> Self {
        self.headers_separator = separator;
        self
    }
    pub fn rseparator(mut self, separator: Option<Separator>) -> Self {
        self.rows_separator = separator;
        self
    }
    pub fn width(mut self, width: impl Into<Width>) -> Self {
        self.width = width.into();
        self
    }

    pub fn build(self) -> FancyTable<'a, T> {
        let width = match self.width {
            Width::Fixed(w) => w,
            Width::Percentage(pct) => {
                use terminal_size::{Width as TermWidth, terminal_size};
                terminal_size()
                    .map(|(TermWidth(w), _)| w as usize * pct as usize / 100)
                    .unwrap_or(80)
            }
        }
        .max(3);
        let title = self.title.map(|t| TitleSpec {
            title: t,
            align: self.title_align,
        });
        let mut table = FancyTable {
            width,
            chars: self.charset.get_chars(),
            rows_separator: self.rows_separator,
            headers_separator: self.headers_separator,
            padding: self.padding,
            headers: self.headers,
            columns: self.columns,
            title,
        };
        table.recalculate(width);
        table
    }
}

impl<'a, T: AsRef<str>> FancyTable<'a, T> {
    pub fn create(opts: FancyTableOpts) -> FancyTableBuilder<'a, T> {
        FancyTableBuilder::new(opts)
    }

    fn recalculate(&mut self, table_width: usize) {
        let cols_count = self.columns.len();
        let mut min_table_width = 0;

        // calculate minimal table width with all paddings counted in
        for (i, spec) in self.columns.iter_mut().enumerate() {
            let column_width = match spec.layout {
                Layout::Fixed(width) => width,
                Layout::Slim | Layout::Expandable(_) => self
                    .headers
                    .get(i)
                    .map(|h| h.as_ref().chars().count() + (2 * self.padding))
                    .unwrap_or(0),
            };
            spec.width = column_width.max(1);
            min_table_width += spec.width;
        }

        min_table_width += cols_count + 1;

        // adjust columns widths so, that they will all sum up to desired `table_width`
        // by calculating remaining width and distributing it equally (as much as possible)
        // among all expandable columns.
        let mut remaining_width = table_width.saturating_sub(min_table_width);

        if remaining_width > 0 {
            let expandables = self
                .columns
                .iter_mut()
                .filter(|c| matches!(c.layout, Layout::Expandable(_)));

            let mut spec_refs = expandables.collect::<Vec<_>>();
            let mut expandables_count = spec_refs.len();

            // To avoid the situation where expandable columns cannot expand enough to fully fit
            // remaining space the idea is to sort them by max expand widths and oversize only
            // last (longest) column if needed, ie. when requested table width is still bigger
            // than a sum of particular column sizes.

            spec_refs.sort_by_key(|c| match c.layout {
                Layout::Expandable(max) => max,
                _ => c.width,
            });

            for c in spec_refs.into_iter() {
                if let Layout::Expandable(max_expand) = c.layout {
                    let new_width =
                        min(c.width + (remaining_width / expandables_count), max_expand);
                    let compensation = new_width.saturating_sub(c.width);

                    // Oversize biggest expandable column in case when there is still
                    // some remaining space but no more expandable columns to expand.
                    if expandables_count == 1 {
                        c.width += remaining_width;
                    } else if compensation > 0 {
                        c.width = new_width;
                        remaining_width -= compensation;
                    }
                    expandables_count -= 1;
                }
            }
        }
    }

    fn generate_empty_string(&self, col_idx: usize, padding: usize) -> String {
        if let Some(col) = self.columns.get(col_idx) {
            let width = col.width.saturating_sub(2 * padding);
            let mut result = String::with_capacity(width);
            result.push_str(&" ".repeat(width));
            return result;
        }
        String::default()
    }

    fn separator_chars(&self, separator: &Option<Separator>) -> (char, char, char, char) {
        let ch = &self.chars;
        match separator {
            Some(Separator::Single) => (ch.ew, ch.news, ch.nes, ch.nws),
            Some(Separator::Double) => (ch.dew, ch.dnews, ch.dnes, ch.dnws),
            Some(Separator::Custom(c)) => (*c, ch.news, ch.nes, ch.nws),
            None => ('-', '|', '|', '|'),
        }
    }

    fn render_row(&self, row: &'a [T]) {
        let mut padded = row
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let col = self.columns.get(i).unwrap();
                let pad = match col.align {
                    Align::Left => Justify::Left,
                    Align::Right => Justify::Right,
                    Align::Center => Justify::Center,
                };
                match col.overflow {
                    Overflow::Truncate => JustedString::truncating(s.as_ref()),
                    Overflow::Wrap => JustedString::wrapping(s.as_ref()),
                }
                .justify(
                    col.width.saturating_sub(2 * self.padding),
                    col.max_lines,
                    pad,
                )
            })
            .collect::<Vec<_>>();

        let ns = self.chars.ns;
        let len = padded.len();
        let max_lines = padded.iter().map(|s| s.len()).max().unwrap_or(0);
        let str_padding = self.padding;
        let edg_padding = self.padding + 1;

        for _ in 0..max_lines {
            print!("{:edg_padding$}", ns);
            for (i, vs) in padded.iter_mut().enumerate() {
                let s = vs
                    .pop_front()
                    .unwrap_or_else(|| self.generate_empty_string(i, str_padding));
                print!("{s}");
                if i < len - 1 {
                    print!("{:>str_padding$}{ns}{:>str_padding$}", "", "");
                }
            }
            println!("{:>edg_padding$}", ns);
        }
    }

    pub fn render<R: AsRef<[T]>>(&self, rows: Vec<R>) {
        let ch = &self.chars;
        let cols_count = self.columns.len();
        let rows_count = rows.len();
        let rsep_chars = self.separator_chars(&self.rows_separator);
        let hsep_chars = self.separator_chars(&self.headers_separator);
        // Parse the title through the same ANSI-aware machinery used for cell
        // content, so escape codes count as zero-width and don't throw off
        // the border layout.
        let title_line = self.title.as_ref().and_then(|ts| {
            ansi::build_string(ts.title, self.width, 1, &AnsiOverflow::Truncate)
                .into_iter()
                .next()
        });
        let title_width = title_line.as_ref().map(|tl| tl.len + 4).unwrap_or(0); // decorators on both sides

        let mut acc = 1;
        let mut border_top = vec![ch.ew; self.width];
        let mut border_btm = vec![ch.ew; self.width];
        let mut hseparator = vec![hsep_chars.0; self.width];
        let mut rseparator = vec![rsep_chars.0; self.width];

        border_top[0] = ch.se;
        border_btm[0] = ch.ne;
        border_top[self.width - 1] = ch.sw;
        border_btm[self.width - 1] = ch.nw;

        hseparator[0] = hsep_chars.2;
        rseparator[0] = rsep_chars.2;
        hseparator[self.width - 1] = hsep_chars.3;
        rseparator[self.width - 1] = rsep_chars.3;

        // prepare top and bottom lines.
        for (i, spec) in self.columns.iter().enumerate() {
            if i < cols_count - 1 {
                acc += spec.width + 1;
                border_top[acc - 1] = ch.ews;
                border_btm[acc - 1] = ch.new;
                hseparator[acc - 1] = hsep_chars.1;
                rseparator[acc - 1] = rsep_chars.1;
            }
        }

        // draw a title
        if title_width > 0 && title_width < self.width - 4 {
            let spec = self.title.as_ref().unwrap();
            let tl = title_line.as_ref().unwrap();
            let start = match spec.align {
                TitleAlign::LeftOffset(lo) => lo + 1,
                TitleAlign::RightOffset(ro) => self.width - ro - title_width - 1,
            };
            let end = start + title_width;
            let tch = ch.title;

            let mut decorated = String::with_capacity(tl.slice.len() + 6);
            decorated.push(tch);
            decorated.push(' ');
            if let Some(c2c) = &tl.c2c {
                decorated.push_str(c2c);
            }
            decorated.push_str(tl.slice);
            if tl.needs_rst {
                decorated.push_str(ansi::RST_CODE);
            }
            decorated.push(' ');
            decorated.push(tch);

            border_top.splice(start..end, decorated.chars());
        }

        let top = border_top.iter().collect::<String>();
        let btm = border_btm.iter().collect::<String>();
        let h_sep = hseparator.iter().collect::<String>();
        let r_sep = rseparator.iter().collect::<String>();

        println!("{top}");
        if !self.headers.is_empty() {
            self.render_row(self.headers.as_slice());
            if self.headers_separator.is_some() {
                println!("{h_sep}");
            }
        }
        for (i, r) in rows.iter().enumerate() {
            self.render_row(r.as_ref());
            if i < rows_count - 1 && self.rows_separator.is_some() {
                println!("{r_sep}");
            }
        }
        println!("{btm}");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_constraints() {
        let table = FancyTable::create(FancyTableOpts::default())
            .add_column_named("ID", Layout::Fixed(8))
            .add_column_named("NAME", Layout::Fixed(4))
            .add_column_named("ROLE", Layout::Fixed(10))
            .add_column_named("PERMISSION", Layout::Expandable(30))
            .add_column_named("DESCRIPTION", Layout::Expandable(150))
            .add_title("props")
            .padding(0)
            .build();

        assert_eq!(table.columns.first().unwrap().width, 8);
        assert_eq!(table.columns.get(1).unwrap().width, 4);
        assert_eq!(table.columns.get(2).unwrap().width, 10);
        assert_eq!(table.columns.get(3).unwrap().width, 25);
        assert_eq!(
            table.columns.get(4).unwrap().width,
            80 - 6 - 8 - 4 - 10 - 25
        );
    }

    #[test]
    fn slim_table() {
        let table = FancyTable::create(FancyTableOpts::default())
            .add_column_named("ID", Layout::Slim)
            .add_column_named("NAME", Layout::Slim)
            .add_column_named("ROLE", Layout::Fixed(10))
            .add_column_named("PERMISSION", Layout::Expandable(30))
            .add_column_named("DESCRIPTION", Layout::Expandable(50))
            .padding(0)
            .width(0)
            .build();

        assert_eq!(table.columns.first().unwrap().width, 2);
        assert_eq!(table.columns.get(1).unwrap().width, 4);
        assert_eq!(table.columns.get(2).unwrap().width, 10);
        assert_eq!(table.columns.get(3).unwrap().width, 10);
        assert_eq!(table.columns.get(4).unwrap().width, 11);
    }

    #[test]
    fn minimum_column_width() {
        // Fixed(0) and an empty header should both floor to 1
        let table = FancyTable::create(FancyTableOpts::default())
            .add_column_named("", Layout::Slim)
            .add_column_named("X", Layout::Fixed(0))
            .padding(0)
            .build();

        assert_eq!(table.columns.first().unwrap().width, 1);
        assert_eq!(table.columns.get(1).unwrap().width, 1);
    }
}
