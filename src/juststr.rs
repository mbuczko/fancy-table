use std::collections::VecDeque;

use crate::{ansi, ansi::AnsiString};

#[derive(Debug, Clone)]
pub enum Justify {
    Left,
    Right,
    Center,
}

pub struct JustedString<'a> {
    input: &'a str,
    overflow: ansi::Overflow,
}

impl<'a> JustedString<'a> {
    pub fn truncating(input: &'a str) -> Self {
        Self {
            input,
            overflow: ansi::Overflow::Truncate,
        }
    }

    pub fn wrapping(input: &'a str) -> Self {
        Self {
            input,
            overflow: ansi::Overflow::WordWrap,
        }
    }

    /// Processes input text into justified lines that fit within specified dimensions.
    ///
    /// This is the main entry point for text justification. It first processes the input
    /// text to handle ANSI codes and line breaking, then applies the specified alignment
    /// to each resulting line.
    ///
    /// # Arguments
    /// * `hspace` - Maximum characters per line (excluding ANSI codes)
    /// * `vspace` - Maximum number of lines to generate
    /// * `pad` - Text alignment strategy (left, right, center)
    ///
    /// # Returns
    /// A queue of fully formatted strings, each representing a justified line
    pub fn justify(&self, hspace: usize, vspace: usize, pad: Justify) -> VecDeque<String> {
        let ansi_strings = ansi::build_string(self.input, hspace, vspace, &self.overflow);

        ansi_strings
            .into_iter()
            .map(|ansi_string| self.wrap_with_paddings(ansi_string, hspace, &pad))
            .collect::<VecDeque<_>>()
    }

    /// Formats a single line of text with proper alignment and ANSI code handling.
    ///
    /// Takes an ANSI-formatted string and applies the specified justification within
    /// the given horizontal space, ensuring proper padding and ANSI code continuation.
    ///
    /// # Arguments
    /// * `astr` - The ANSI string containing text, formatting codes, and metadata
    /// * `hspace` - Total horizontal space available for the formatted text
    /// * `pad` - Justification strategy (left, right, center)
    ///
    /// # Returns
    /// A properly formatted string with alignment padding and ANSI codes
    fn wrap_with_paddings(&self, astr: AnsiString, hspace: usize, pad: &Justify) -> String {
        let slice = astr.slice;
        let needs_padding = hspace - astr.len;

        if needs_padding == 0 && astr.c2c.is_none() && !astr.needs_rst {
            return astr.slice.to_string();
        }

        let mut result = String::with_capacity(
            astr.c2c.as_ref().map(|s| s.len()).unwrap_or(0)
                + astr.len
                + needs_padding
                + (astr.needs_rst as usize * ansi::RST_CODE.len()),
        );
        if let Some(c2c) = astr.c2c {
            result.push_str(&c2c);
        }
        match pad {
            Justify::Left => {
                result.push_str(slice);
                for _ in 0..needs_padding {
                    result.push(' ');
                }
            }
            Justify::Right => {
                for _ in 0..needs_padding {
                    result.push(' ');
                }
                result.push_str(slice);
            }
            Justify::Center => {
                let left_padding = needs_padding / 2;
                let right_padding = needs_padding - left_padding;

                for _ in 0..left_padding {
                    result.push(' ');
                }
                result.push_str(slice);
                for _ in 0..right_padding {
                    result.push(' ');
                }
            }
        }
        if astr.needs_rst {
            result.push_str(ansi::RST_CODE);
        }
        result
    }
}
