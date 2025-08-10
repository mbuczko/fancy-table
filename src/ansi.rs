use std::{borrow::Cow, cmp::min, fmt};

use regex::Regex;

#[derive(Debug)]
enum MatchLike<'t> {
    Real(regex::Match<'t>),
    Synth(usize),
}

impl<'t> MatchLike<'t> {
    fn start(&self) -> usize {
        match self {
            Self::Real(m) => m.start(),
            Self::Synth(start) => *start,
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::Real(m) => m.len(),
            Self::Synth(_) => 0,
        }
    }
    fn as_str(&self) -> &'t str {
        match self {
            Self::Real(m) => m.as_str(),
            Self::Synth(_) => "",
        }
    }
}

#[derive(Debug)]
/// Representation of a single segment of text starting with
/// optional SGR code, optionally reset at the end of string.
///
/// Example: "\x1b[38;2;255;105;180mHot Pink\x1b[0m"
pub struct AnsiSegment<'a> {
    pub sgr_code: Option<Cow<'a, str>>,
    pub rst_code: Option<Cow<'a, str>>,
    pub text: &'a str,
}

#[derive(Debug, Default)]
pub struct AnsiString<'a> {
    len: usize,
    segments: Vec<AnsiSegment<'a>>,
}

impl<'a> fmt::Display for AnsiSegment<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let sgr = self.sgr_code.as_deref().unwrap_or("");
        let rst = self.rst_code.as_deref().unwrap_or("");
        write!(f, "{}{}{})", sgr, self.text, rst)
    }
}

impl<'a> AnsiSegment<'a> {
    pub fn len(&self) -> usize {
        self.text.chars().count()
    }
    pub fn size(&self) -> usize {
        self.text.len() + self.sgr_size() + self.rst_size()
    }
    pub fn has_sgr_code(&self) -> bool {
        self.sgr_code.is_some()
    }
    pub fn has_rst_code(&self) -> bool {
        self.rst_code.is_some()
    }
    pub fn sgr_size(&self) -> usize {
        self.sgr_code.as_ref().map(|c| c.len()).unwrap_or(0)
    }
    pub fn rst_size(&self) -> usize {
        self.rst_code.as_ref().map(|c| c.len()).unwrap_or(0)
    }
    pub fn is_sgr_owned(&self) -> bool {
        matches!(self.sgr_code, Some(Cow::Owned(_)))
    }
}

impl<'a> AnsiString<'a> {
    pub fn new(input: &'a str) -> Self {
        let regex = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        build_ansi_string(regex, input)
    }
    pub fn with_sgr(mut self, codes: String) -> Self {
        if let Some(seg) = self.segments.first_mut() {
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
    pub fn append(&mut self, str: AnsiString<'a>) {
        self.len += str.len() + !self.is_empty() as usize;
        self.segments.extend(str.segments);
    }
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn push_segment(&mut self, segment: AnsiSegment<'a>) {
        self.len += segment.len() + !self.is_empty() as usize;
        self.segments.push(segment);
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

    pub fn get(&self, len: usize) -> Cow<'a, str> {
        let (_, bytes, _needs_rst) =
            self.segments
                .iter()
                .fold((0, 0, true), |(text_len, byte_size, is_reset), segment| {
                    if text_len >= len {
                        (text_len, byte_size, is_reset)
                    } else {
                        let slen = text_len + segment.len();
                        let diff = text_len.saturating_sub(len);
                        (
                            min(slen, len),
                            byte_size
                                + 1
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

            return if let Some(sgr) = seg.sgr_code.as_ref()
                && seg.is_sgr_owned()
            {
                Cow::Owned(format!("{sgr}{str}"))
            } else {
                Cow::Borrowed(str)
            };
        }
        Cow::Borrowed("")
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

fn is_rst(code: &str) -> bool {
    code == "\x1b[0m"
}

fn build_ansi_string<'a>(regex: Regex, input: &'a str) -> AnsiString<'a> {
    let mut result = AnsiString::default();
    let mut last_code = (0, 0, false); // pos, len, is_reset?

    for mat in regex
        .find_iter(input)
        .map(MatchLike::Real)
        .chain(std::iter::once(MatchLike::Synth(input.len())))
    {
        let code_start = mat.start();
        let code_len = mat.len();
        let is_reset = is_rst(mat.as_str());

        let (last_code_pos, last_code_len, last_code_is_reset) = last_code;
        let last_code_end = last_code_pos + last_code_len;

        last_code = if last_code_end < code_start {
            result.push_segment(AnsiSegment {
                text: &input[last_code_end..code_start],
                sgr_code: if !last_code_is_reset && last_code_len > 0 {
                    Some(Cow::Borrowed(&input[last_code_pos..last_code_end]))
                } else {
                    None
                },
                rst_code: if is_reset {
                    Some(Cow::Borrowed(mat.as_str()))
                } else {
                    None
                },
            });
            (code_start, code_len, is_reset)
        } else if last_code_is_reset {
            (code_start, code_len, is_reset)
        } else {
            (last_code_pos, last_code_len + code_len, is_reset)
        }
    }
    result
}

#[macro_export]
macro_rules! assert_codes {
        ($string:expr, [$($segment:tt),+]) => {
            {
                let str = format!($string);
                let ansi = $crate::AnsiString::new(&str);
                let segments = ansi.segments();
                let mut segment_index = 0;

                $(
                    assert_codes!(@verify_segment segments[segment_index], $segment);
                    segment_index += 1;
                )+
                    assert_eq!(segments.len(), segment_index, "Expected {} segments, found {}", segment_index, segments.len());
            }
        };
        (@verify_segment $seg:expr, { $($field:ident => $value:tt),* }) => {
            let seg = &$seg;
            $(
                assert_codes!(@check_field seg, $field, $value);
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
