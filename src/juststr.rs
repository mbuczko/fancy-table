use std::{collections::VecDeque, ops::Deref};

use crate::ansi::AnsiString;

#[derive(Debug)]
enum Chunk<'a> {
    Word(&'a str),
    Term(&'a str),
}

#[derive(Clone)]
pub enum Justify {
    Left,
    Right,
    Center,
}

pub struct JustedString<'a> {
    chunks: Vec<Chunk<'a>>,
}

fn should_wrap(agg: &AnsiString, str: &AnsiString, hspace: usize) -> bool {
    let line_start = agg.is_empty();
    !line_start && (agg.len() + (!line_start as usize) + str.len() > hspace)
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

impl<'a> JustedString<'a> {
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

    pub fn justify(&self, hspace: usize, vspace: usize, pad: Justify) -> VecDeque<String> {
        let mut bag = VecDeque::with_capacity(vspace);
        let mut agg = AnsiString::default();
        let mut c2c = None;

        let count = self.chunks.len();

        for (i, chunk) in self.chunks.iter().enumerate() {
            let is_last_str = i == count - 1;
            let is_term_str = matches!(chunk, Chunk::Term(_));
            let ansi_string = AnsiString::new(chunk).with_sgr(c2c);

            c2c = None;

            if should_wrap(&agg, &ansi_string, hspace) {
                // When text wraps to the next line, any active formatting codes
                // should be reapplied at the start of the new line.
                let c2c = agg.codes_to_continue();

                bag.push_back(self.wrap_with_padding(agg, hspace, &pad));
                agg = ansi_string.with_sgr(Some(c2c))
            } else {
                agg.append(ansi_string)
            }
            if bag.len() < vspace && (agg.len() == hspace || is_last_str || is_term_str) {
                // Enforced line termination or reaching end of allowed space
                // also requires formatting code to be reapplied in new line.
                c2c = Some(agg.codes_to_continue());

                bag.push_back(self.wrap_with_padding(agg, hspace, &pad));
                agg = AnsiString::default();
            }
            if bag.len() == vspace {
                return bag;
            }
        }
        bag
    }

    /// Justifies text within the given horizontal space, accounting for ANSI escape codes.
    fn wrap_with_padding(&self, s: AnsiString, hspace: usize, pad: &Justify) -> String {
        let slice = s.get(hspace);
        let text = &*slice;
        let text_len = slice.len;

        if text_len >= hspace {
            return slice.owned();
        }

        let padding_needed = hspace - text_len;

        match pad {
            Justify::Left => {
                let mut result = String::with_capacity(text.len() + padding_needed);
                result.push_str(text);
                for _ in 0..padding_needed {
                    result.push(' ');
                }
                result
            }
            Justify::Right => {
                let mut result = String::with_capacity(text.len() + padding_needed);
                for _ in 0..padding_needed {
                    result.push(' ');
                }
                result.push_str(text);
                result
            }
            Justify::Center => {
                let left_padding = padding_needed / 2;
                let right_padding = padding_needed - left_padding;

                let mut result = String::with_capacity(text.len() + padding_needed);
                for _ in 0..left_padding {
                    result.push(' ');
                }
                result.push_str(text);
                for _ in 0..right_padding {
                    result.push(' ');
                }
                result
            }
        }
    }
}
