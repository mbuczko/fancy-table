use crate::{
    charset::Charset,
    padstr::{Pad, PadStr},
    Align, ColSpec, FancyTable, FancyTableBuilder, FancyTableOpts, Layout, Overflow, Separator,
    TitleAlign, TitleSpec,
};

const DEFAULT_WIDTH: u16 = 120;

impl Default for FancyTableOpts {
    fn default() -> Self {
        Self {
            title_align: TitleAlign::LeftOffset(4),
            charset: Charset::Modern,
            headers_separator: Some(Separator::Double),
            rows_separator: Some(Separator::Single),
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
            charset: opts.charset,
            rows_separator: opts.rows_separator,
            headers_separator: opts.headers_separator,
            max_lines: opts.max_lines,
            title: None,
            title_align: opts.title_align,
        }
    }
    pub fn add_column(
        mut self,
        width: usize,
        max_lines: usize,
        layout: Layout,
        align: Align,
        overflow: Overflow,
    ) -> Self {
        self.columns.push(ColSpec {
            width,
            layout,
            align,
            overflow,
            max_lines,
        });
        self
    }
    pub fn add_column_named(self, header: T, layout: Layout) -> Self {
        self.add_column_named_with_align(header, layout, Align::Left)
    }
    pub fn add_wrapping_column_named(self, header: T, layout: Layout) -> Self {
        self.add_wrapping_column_named_with_align(header, layout, Align::Left)
    }
    pub fn add_column_named_with_align(
        mut self,
        header: T,
        layout: Layout,
        align: Align,
    ) -> Self {
        let len = header.as_ref().len();
        let max_lines = self.max_lines;

        self.headers.push(header);
        self.add_column(len, max_lines, layout, align, Overflow::Truncate)
    }
    pub fn add_wrapping_column_named_with_align(
        mut self,
        header: T,
        layout: Layout,
        align: Align,
    ) -> Self {
        let len = header.as_ref().len();
        let max_lines = self.max_lines;

        self.headers.push(header);
        self.add_column(len, max_lines, layout, align, Overflow::Wrap)
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

    pub fn build(self, table_width: usize) -> FancyTable<'a, T> {
        let title = self.title.map(|t| TitleSpec {
            title: t,
            align: self.title_align,
        });
        let mut table = FancyTable {
            width: table_width,
            chars: self.charset.get_chars(),
            rows_separator: self.rows_separator,
            headers_separator: self.headers_separator,
            padding: self.padding,
            headers: self.headers,
            columns: self.columns,
            title,
        };
        table.recalculate(table_width);
        table
    }
    pub fn build_with_max_width(self) -> FancyTable<'a, T> {
        let w = match termion::terminal_size() {
            Ok((w, _)) => w,
            _ => DEFAULT_WIDTH,
        };
        self.build(w as usize)
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
                    .map(|h| h.as_ref().len() + (2 * self.padding))
                    .unwrap_or(0),
            };
            spec.width = column_width;
            min_table_width += spec.width;
        }

        min_table_width += cols_count + 1;

        // adjust columns widths so, that they will all sum up to desired `table_width`
        // by calculating remaining width and distributing it equally (as much as possible)
        // among all expandable columns.
        let mut remaining_width = table_width.saturating_sub(min_table_width);

        if remaining_width > 0 {
            let expandable_cols = self
                .columns
                .iter_mut()
                .filter_map(|c| match c.layout {
                    Layout::Expandable(max_width) => Some((c, max_width)),
                    _ => None,
                })
                .collect::<Vec<_>>();

            let mut expandable_count = expandable_cols.len();
            for (ec, max_width) in expandable_cols {
                let new_width = compensate(ec.width, max_width, remaining_width / expandable_count);
                let compensation = new_width.saturating_sub(ec.width);

                if new_width > ec.width {
                    ec.width = new_width;
                }
                remaining_width -= compensation;
                expandable_count -= 1;
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
                    Align::Left => Pad::Right,
                    Align::Right => Pad::Left,
                    Align::Center => Pad::Center,
                };
                match col.overflow {
                    Overflow::Truncate => PadStr::truncating(s.as_ref()),
                    Overflow::Wrap => PadStr::wrapping(s.as_ref()),
                }
                .paddify(
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
        let title_width = self
            .title
            .as_ref()
            .map(|ts| ts.title.len() + 4)
            .unwrap_or(0);

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
            let start = match spec.align {
                TitleAlign::LeftOffset(lo) => lo + 1,
                TitleAlign::RightOffset(ro) => self.width - ro - title_width - 1,
            };
            let end = start + title_width;
            let tch = ch.title;
            border_top.splice(start..end, format!("{tch} {} {tch}", spec.title).chars());
        }

        let top = border_top.iter().collect::<String>();
        let btm = border_btm.iter().collect::<String>();
        let h_sep = hseparator.iter().collect::<String>();
        let r_sep = rseparator.iter().collect::<String>();

        println!("{top}");
        if !self.headers.is_empty() {
            self.render_row(self.headers.as_slice());
        }
        if self.headers_separator.is_some() {
            println!("{h_sep}");
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

fn compensate(width: usize, max_width: usize, compensation: usize) -> usize {
    let compensated = width + compensation;
    if compensated > max_width {
        max_width
    } else {
        compensated
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
            .build(80);

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
            .build(0);

        assert_eq!(table.columns.first().unwrap().width, 2);
        assert_eq!(table.columns.get(1).unwrap().width, 4);
        assert_eq!(table.columns.get(2).unwrap().width, 10);
        assert_eq!(table.columns.get(3).unwrap().width, 10);
        assert_eq!(table.columns.get(4).unwrap().width, 11);
    }
}
