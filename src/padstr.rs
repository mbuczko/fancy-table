use std::{collections::VecDeque, ops::Deref};

use crate::ansi::AnsiString;

#[derive(Debug)]
enum Chunk<'a> {
    Word(&'a str),
    Term(&'a str),
}

#[derive(Clone)]
pub enum Pad {
    Left,
    Right,
    Center,
}

pub struct PadStr<'a> {
    chunks: Vec<Chunk<'a>>,
}

fn should_wrap(agg: &AnsiString, str: &AnsiString, hspace: usize) -> bool {
    let line_start = agg.is_empty();
    !line_start && (agg.len() + (!line_start as usize) + str.len() > hspace)
}

fn center_string(s: &str, width: usize) -> String {
    let padding = width - s.chars().count();
    let left_padding = padding / 2;
    let right_padding = padding - left_padding;

    format!("{:>left_padding$}{s}{:>right_padding$}", "", "")
}

fn rightpad_string(s: &str, width: usize) -> String {
    format!("{s:width$}")
}

fn leftpad_string(s: &str, width: usize) -> String {
    format!("{s:>width$}")
}

impl<'a> AsRef<str> for Chunk<'a> {
    fn as_ref(&self) -> &str {
        match self {
            Self::Word(s) | Self::Term(s) => s,
        }
    }
}

impl<'a> Deref for Chunk<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<'a> PadStr<'a> {
    pub fn truncating(input: &'a str) -> Self {
        let chunks = input.lines().map(Chunk::Term).collect::<Vec<_>>();
        Self { chunks }
    }

    pub fn wrapping(input: &'a str) -> Self {
        let chunks = input
            .lines()
            .flat_map(|l| {
                let mut iter = l.split(' ').peekable();

                std::iter::from_fn(move || {
                    iter.next().map(|slice| {
                        if iter.peek().is_none() {
                            Chunk::Term(slice)
                        } else {
                            Chunk::Word(slice)
                        }
                    })
                })
            })
            .collect::<Vec<_>>();

        Self { chunks }
    }

    pub fn paddify(&self, hspace: usize, vspace: usize, pad: Pad) -> VecDeque<String> {
        let mut bag = VecDeque::with_capacity(vspace);
        let mut agg = AnsiString::default();
        let count = self.chunks.len();

        for (i, chunk) in self.chunks.iter().enumerate() {
            let is_last_str = i == count - 1;
            let is_term_str = matches!(chunk, Chunk::Term(_));
            let ansi_string = AnsiString::new(chunk);

            if should_wrap(&agg, &ansi_string, hspace) {
                // When text wraps to the next line, any active formatting codes
                // should be reapplied at the start of the new line.
                let c2c = agg.codes_to_continue();

                bag.push_back(self.wrap_with_padding(agg, hspace, &pad));
                agg = ansi_string.with_sgr(c2c)
            } else {
                agg.append(ansi_string)
            }
            if bag.len() < vspace && (agg.len() == hspace || is_last_str || is_term_str) {
                bag.push_back(self.wrap_with_padding(agg, hspace, &pad));
                agg = AnsiString::default();
            }
            if bag.len() == vspace {
                return bag;
            }
        }
        bag
    }

    fn wrap_with_padding(&self, s: AnsiString, hspace: usize, pad: &Pad) -> String {
        let slice = s.get(hspace);
        match pad {
            Pad::Left => leftpad_string(&slice, hspace),
            Pad::Right => rightpad_string(&slice, hspace),
            Pad::Center => center_string(&slice, hspace),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn justify_center_fit_hspace() {
        let js = PadStr::wrapping("Ala ma kota");
        let lines = js.paddify(6, 2, Pad::Center);
        assert_eq!(lines, vec!["Ala ma", " kota "]);
    }

    #[test]
    fn justify_center_enough_hspace() {
        let js = PadStr::wrapping("Ala ma kota");
        let lines = js.paddify(10, 2, Pad::Center);
        assert_eq!(lines, vec!["  Ala ma  ", "   kota   "]);
    }

    #[test]
    fn justify_center_no_enough_vspace() {
        let js = PadStr::wrapping("Ala ma kota");
        let lines = js.paddify(8, 1, Pad::Center);
        assert_eq!(lines, vec![" Ala ma "])
    }

    #[test]
    fn justify_center_no_enough_hspace() {
        let js = PadStr::wrapping("Ala ma kota");
        let lines = js.paddify(2, 3, Pad::Center);
        assert_eq!(lines, vec!["Al", "ma", "ko"]);
    }

    #[test]
    fn justify_center_no_enough_hspace_and_vspace() {
        let js = PadStr::wrapping("Ala ma kota");
        let lines = js.paddify(1, 2, Pad::Center);
        assert_eq!(lines, vec!["A", "m"]);
    }

    #[test]
    fn justify_left_enough_hspace() {
        let js = PadStr::wrapping("Ala ma kota");
        let lines = js.paddify(10, 2, Pad::Right);
        assert_eq!(lines, vec!["Ala ma    ", "kota      "]);
    }

    #[test]
    fn justify_right_enough_hspace() {
        let js = PadStr::wrapping("Ala ma kota");
        let lines = js.paddify(10, 2, Pad::Left);
        assert_eq!(lines, vec!["    Ala ma", "      kota"]);
    }

    #[test]
    fn justify_right_not_enough_hspace() {
        let js = PadStr::wrapping("Ala ma kota");
        let lines = js.paddify(5, 2, Pad::Left);
        assert_eq!(lines, vec!["  Ala", "   ma"]);
    }

    #[test]
    fn justify_line_with_wrapping() {
        let js = PadStr::wrapping("Ala ma kota\nA kot ma Alę");
        let lines = js.paddify(7, 5, Pad::Right);
        assert_eq!(lines, vec!["Ala ma ", "kota   ", "A kot  ", "ma Alę "]);
    }

    #[test]
    fn justify_line_with_enough_hspace() {
        let js = PadStr::truncating("Ala ma kota\nA kot ma Alę");
        let lines = js.paddify(15, 2, Pad::Left);
        assert_eq!(lines, vec!["    Ala ma kota", "   A kot ma Alę"]);
    }

    #[test]
    fn justify_line_with_not_enough_hspace() {
        let js = PadStr::truncating("Ala ma kota\nA kot ma Alę");
        let lines = js.paddify(8, 2, Pad::Right);
        assert_eq!(lines, vec!["Ala ma k", "A kot ma"]);
    }

    #[test]
    fn justify_line_with_not_enough_hspace_and_vspace() {
        let js = PadStr::truncating("Ala ma kota\nA kot ma Alę\nOna go kocha\nA on ją wcale");
        let lines = js.paddify(8, 3, Pad::Center);
        assert_eq!(lines, vec!["Ala ma k", "A kot ma", "Ona go k"]);
    }
}
