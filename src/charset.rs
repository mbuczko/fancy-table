pub struct TableChars {
    pub se: char,
    pub ew: char,
    pub nw: char,
    pub sw: char,
    pub ns: char,
    pub ne: char,
    pub ews: char,
    pub nes: char,
    pub nws: char,
    pub new: char,
    pub news: char,
    pub dew: char,
    pub dnes: char,
    pub dnws: char,
    pub dnews: char,
    pub title: char,
}

#[allow(dead_code)]
pub enum Charset {
    Classic,
    Modern,
    Simple,
    Minimal,
}

impl Charset {
    pub fn get_chars(&self) -> TableChars {
        match self {
            Self::Modern => TableChars {
                se: '╭',
                nw: '╯',
                sw: '╮',
                ns: '│',
                ne: '╰',
                ew: '─',
                ews: '┬',
                nes: '├',
                nws: '┤',
                new: '┴',
                dew: '═',
                news: '┼',
                dnes: '╞',
                dnws: '╡',
                dnews: '╪',
                title: '▪',
            },
            Self::Classic => TableChars {
                se: '┌',
                nw: '┘',
                sw: '┐',
                ns: '│',
                ne: '└',
                ew: '─',
                ews: '┬',
                nes: '├',
                nws: '┤',
                new: '┴',
                dew: '═',
                news: '┼',
                dnes: '╞',
                dnws: '╡',
                dnews: '╪',
                title: '▪',
            },
            Self::Simple => TableChars {
                se: '+',
                nw: '+',
                sw: '+',
                ns: '|',
                ne: '+',
                ew: '-',
                ews: '+',
                nes: '|',
                nws: '|',
                new: '+',
                dew: '=',
                news: '+',
                dnes: '|',
                dnws: '|',
                dnews: '=',
                title: '*',
            },
            Self::Minimal => TableChars {
                se: ' ',
                nw: ' ',
                sw: ' ',
                ns: ' ',
                ne: ' ',
                ew: '-',
                ews: '-',
                nes: ' ',
                nws: ' ',
                new: '-',
                dew: '=',
                news: '-',
                dnes: ' ',
                dnws: ' ',
                dnews: '=',
                title: '=',
            },
        }
    }
}