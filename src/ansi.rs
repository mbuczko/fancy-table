use std::{borrow::Cow, cmp::min, fmt, ops::Deref};

#[derive(PartialEq)]
enum AnsiToken {
    Escape,
    Opening,
    Code,
}

#[derive(Debug)]
pub struct AnsiSlice<'a> {
    pub slice: Cow<'a, str>,
    pub len: usize,
    pub needs_rst: bool,
}

impl<'a> Deref for AnsiSlice<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.slice.as_ref()
    }
}

impl<'a> PartialEq<&str> for AnsiSlice<'a> {
    fn eq(&self, other: &&str) -> bool {
        self.slice.as_ref() == *other
    }
}

impl<'a> PartialEq<String> for AnsiSlice<'a> {
    fn eq(&self, other: &String) -> bool {
        self.slice.as_ref() == other.as_str()
    }
}

impl<'a> AnsiSlice<'a> {
    pub fn tupled(&'a self) -> (&'a str, usize) {
        (self.slice.as_ref(), self.len)
    }

    pub fn owned(self) -> String {
        match self.slice {
            Cow::Owned(text) => text,
            Cow::Borrowed(text) => text.to_string(),
        }
    }
}

#[derive(Debug)]
/// Representation of a single segment of a String with ANSI codes: an optional opening code (SGR) and a reset code.
/// Each segment contains an optional code only at the starting position and a reset code at the end.
///
/// Example: "\x1b[38;2;255;105;180mHot Pink\x1b[0m"
pub struct AnsiSegment<'a> {
    pub sgr_code: Option<Cow<'a, str>>,
    pub rst_code: Option<Cow<'a, str>>,
    pub text: &'a str,

    /// is this initial segment of the string?
    is_initial: bool,
}

#[derive(Debug, Default)]
/// Represents a string containing ANSI escape codes. The string is internally divided into segments,
/// where each segment represents a portion of the string with its associated opening SGR (Select Graphic Rendition)
/// code and corresponding reset code.
pub struct AnsiString<'a> {
    len: usize,
    segments: Vec<AnsiSegment<'a>>,
}

impl<'a> fmt::Display for AnsiSegment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sgr = self.sgr_code.as_deref().unwrap_or("");
        let rst = self.rst_code.as_deref().unwrap_or("");
        write!(f, "{}{}{}", sgr, self.text, rst)
    }
}

impl<'a> AnsiSegment<'a> {
    pub fn len(&self) -> usize {
        self.text.chars().count() + self.is_initial as usize
    }

    /// Returns the byte size of a segment. The size is calculated based on the following components:
    ///
    ///  - The size of the text (in bytes)
    ///  - The combined size of both SGR (Select Graphic Rendition) and reset codes
    ///  - An initial flag that, if true, adds +1 to the size to account for a separating space
    fn size(&self) -> usize {
        self.text.len() + self.sgr_size() + self.rst_size() + self.is_initial as usize
    }
    fn has_sgr_code(&self) -> bool {
        self.sgr_code.is_some()
    }
    fn has_rst_code(&self) -> bool {
        self.rst_code.is_some()
    }
    /// Returns SGR code size in case of Cow::Borrowed variant.
    /// For Cow::Owned returns 0 as the code cannot be the part of text slice.
    fn sgr_size(&self) -> usize {
        if self.is_sgr_owned() {
            return 0;
        }
        self.sgr_code.as_ref().map(|c| c.len()).unwrap_or(0)
    }
    fn rst_size(&self) -> usize {
        self.rst_code.as_ref().map(|c| c.len()).unwrap_or(0)
    }
    fn is_sgr_owned(&self) -> bool {
        matches!(self.sgr_code, Some(Cow::Owned(_)))
    }
}

impl<'a> AnsiString<'a> {
    pub fn new(input: &'a str) -> Self {
        build_ansi_string(input)
    }
    pub fn with_sgr(mut self, codes: Option<String>) -> Self {
        if let Some(codes) = codes
            && !codes.is_empty()
            && let Some(seg) = self.segments.first_mut()
        {
            seg.sgr_code = Some(Cow::Owned(codes))
        }
        self
    }
    pub fn segments(&self) -> &[AnsiSegment<'a>] {
        &self.segments
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn append(&mut self, mut str: AnsiString<'a>) {
        if !self.segments.is_empty()
            && let Some(seg) = str.segments.first_mut()
        {
            // appended string cannot start with Cow::Owned SGR.
            assert!(!seg.is_sgr_owned());

            seg.is_initial = true;
            self.len += 1;
        }
        self.len += str.len();
        self.segments.extend(str.segments);
    }

    /// Returns the number of visible characters in the string.
    /// ANSI codes are not counted.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns an SGR code (or multiple joined codes) that should be applied
    /// when continuing text formatting on a text break after the last segment
    /// of an AnsiString.
    pub fn codes_to_continue(&self) -> String {
        let codes = self
            .segments
            .iter()
            .rev()
            .take_while(|seg| !seg.has_rst_code())
            .filter_map(|seg| seg.sgr_code.as_ref().map(|c| c.as_ref()))
            .collect::<Vec<_>>();

        codes.join("")
    }

    /// Returns a substring of the specified length while preserving ANSI codes intact - no ANSI code
    /// will be corrupted. For performance reasons, returns a Cow<str> since in most cases the result
    /// will simply be a sub-slice of the AnsiString. The only case when an owned variant is returned
    /// is when the initial SGR code (stored in the first segment) was explicitly set using the
    /// `with_sgr` function during AnsiString creation.
    pub fn get(&self, len: usize) -> AnsiSlice<'a> {
        let (text_len, bytes, is_terminating_rst) =
            self.segments
                .iter()
                .fold((0, 0, true), |(text_len, byte_size, is_reset), segment| {
                    if text_len >= len {
                        (text_len, byte_size, is_reset)
                    } else {
                        let slen = text_len + segment.len();
                        let diff = slen.saturating_sub(len);
                        (
                            min(slen, len),
                            byte_size
                                + if diff == 0 {
                                    segment.size()
                                } else {
                                    segment.sgr_size()
                                        + segment
                                            .text
                                            .char_indices()
                                            .nth(segment.len() - diff)
                                            .map_or(segment.text.len(), |(byte_idx, _)| byte_idx)
                                },
                            (segment.has_rst_code() && diff == 0)
                                || (is_reset && !segment.has_sgr_code()),
                        )
                    }
                });

        if let Some((ptr, seg)) = self.slice_ptr() {
            let str =
                unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr, bytes)) };

            return AnsiSlice {
                slice: if let Some(sgr) = seg.sgr_code.as_ref()
                    && seg.is_sgr_owned()
                {
                    Cow::Owned(format!("{sgr}{str}"))
                } else {
                    Cow::Borrowed(str)
                },
                len: text_len,
                needs_rst: !is_terminating_rst,
            };
        }
        AnsiSlice {
            slice: Cow::Borrowed(""),
            len: 0,
            needs_rst: false,
        }
    }

    fn push_segment(&mut self, segment: AnsiSegment<'a>) {
        self.len += segment.len() + !self.is_empty() as usize;
        self.segments.push(segment);
    }

    fn slice_ptr(&self) -> Option<(*const u8, &AnsiSegment<'a>)> {
        if let Some(seg) = self.segments.first() {
            return if !seg.has_sgr_code() || seg.is_sgr_owned() {
                Some((seg.text.as_ptr(), seg))
            } else {
                seg.sgr_code.as_ref().map(|c| (c.as_ptr(), seg))
            };
        }
        None
    }
}

pub fn build_ansi_string<'a>(input: &'a str) -> AnsiString<'a> {
    let mut result = AnsiString::default();
    let mut expected = AnsiToken::Escape;
    let mut last_code = (0, 0, false); // start, end, is_reset
    let mut sequence;

    let mut current_code_start = 0;
    let mut text_byte_size: usize = 0;

    for (pos, ch) in input.char_indices() {
        text_byte_size += ch.len_utf8();
        match ch {
            '\x1b' if expected == AnsiToken::Escape => {
                expected = AnsiToken::Opening;
                current_code_start = pos;
            }
            '[' if expected == AnsiToken::Opening => expected = AnsiToken::Code,
            'm' if expected == AnsiToken::Code => {
                // Valid SGR sequence terminator
                sequence = &input[current_code_start..pos + 1];
                text_byte_size = text_byte_size.saturating_sub(sequence.len());

                let is_reset = sequence == "\x1b[0m";
                let is_text = text_byte_size > 0;
                let (last_code_start, last_code_end, was_reset) = last_code;

                // Chunk of text found
                if is_text {
                    result.push_segment(AnsiSegment {
                        text: &input[last_code_end..last_code_end + text_byte_size],
                        sgr_code: if !was_reset && last_code_end > last_code_start {
                            Some(Cow::Borrowed(&input[last_code_start..last_code_end]))
                        } else {
                            None
                        },
                        rst_code: if is_reset {
                            Some(Cow::Borrowed(sequence))
                        } else {
                            None
                        },
                        is_initial: false,
                    });
                }
                last_code = (
                    if is_text || was_reset {
                        current_code_start
                    } else {
                        last_code_start
                    },
                    pos + 1,
                    is_reset,
                );
                text_byte_size = 0;
                expected = AnsiToken::Escape
            }
            '0'..='9' | ';' | ':' if expected == AnsiToken::Code => {
                continue;
            }
            _ => {
                // Invalid character - this is not a valid SGR sequence
            }
        }
    }

    // Final text block not ended with a code.
    // Note, input might be just an empty string. This is to handle this case too.
    if text_byte_size > 0 || input.is_empty() {
        let seg = AnsiSegment {
            sgr_code: if !last_code.2 && last_code.0 != last_code.1 {
                Some(Cow::Borrowed(&input[last_code.0..last_code.1]))
            } else {
                None
            },
            rst_code: None,
            text: &input[last_code.1..],
            is_initial: false,
        };
        result.push_segment(seg)
    }
    result
}

#[macro_export]
macro_rules! assert_segments {
        ($string:expr, [$($segment:tt),+]) => {
            {
                let str = format!($string);
                let ansi = $crate::AnsiString::new(&str);
                let segments = ansi.segments();
                let mut segment_index = 0;

                $(
                    assert_segments!(@verify_segment segments[segment_index], $segment);
                    segment_index += 1;
                )+
                    assert_eq!(segments.len(), segment_index, "Expected {} segments, found {}", segment_index, segments.len());
            }
        };
        (@verify_segment $seg:expr, { $($field:ident => $value:tt),* }) => {
            let seg = &$seg;
            $(
                assert_segments!(@check_field seg, $field, $value);
            )*
        };
        (@check_field $seg:expr, len, $expected:expr) => {
            assert_eq!($seg.len(), $expected);
        };
        (@check_field $seg:expr, txt, $expected:expr) => {
            assert_eq!($seg.text, $expected);
        };
        (@check_field $seg:expr, sgr, $expected:literal) => {
            let formatted = format!($expected);
            assert_eq!($seg.sgr_code, Some(std::borrow::Cow::Borrowed(formatted.as_str())));
        };
        (@check_field $seg:expr, sgr, None) => {
            assert_eq!($seg.sgr_code, None)
        };
        (@check_field $seg:expr, rst, $expected:literal) => {
            let formatted = format!($expected);
            assert_eq!($seg.rst_code, Some(std::borrow::Cow::Borrowed(formatted.as_str())));
        };
        (@check_field $seg:expr, rst, None) => {
            assert_eq!($seg.rst_code, None)
        };
    }
