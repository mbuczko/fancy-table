use std::cmp::min;

pub const RST_CODE: &str = "\x1b[0m";

// open SGRs, text slice, total length (excluding ANSI codes), has reset code?, needs reset?
type AnsiSegment<'a> = (Vec<&'a str>, &'a str, usize, bool, bool);

#[derive(Default)]
pub struct AnsiString<'a> {
    pub slice: &'a str,
    pub c2c: Option<String>,
    pub len: usize,
    pub needs_rst: bool,
}

#[derive(PartialEq)]
enum AnsiToken {
    Escape,
    Opening,
    Code,
}

#[derive(PartialEq)]
pub enum Overflow {
    WordWrap,
    Truncate,
}

#[derive(Debug)]
enum Segment<'a> {
    Word(&'a str, usize),
    Term(&'a str, usize),
}

#[derive(Debug, Default)]
struct CodeQueue<'a> {
    codes_to_continue: Vec<&'a str>,
    codes_to_collect: Vec<&'a str>,
    reset_after_get: bool,
}

impl<'a> CodeQueue<'a> {
    pub fn codes_to_continue(&mut self) -> Option<String> {
        let mut seq = None;
        let count = self.codes_to_continue.len();

        if count > 0 {
            let mut codes = String::with_capacity(count);
            for s in self.codes_to_continue.iter() {
                codes.push_str(s);
            }
            seq = Some(codes)
        }
        if self.reset_after_get {
            self.codes_to_continue.clear();
            self.reset_after_get = false;
        }
        // codes collected become codes to continue in case of line breaks
        self.codes_to_continue.append(&mut self.codes_to_collect);
        seq
    }
    pub fn collect(&mut self, codes: Vec<&'a str>) {
        self.codes_to_collect.extend(codes);
    }
    pub fn clear(&mut self) {
        self.reset_after_get = true;
        self.codes_to_collect.clear();
    }
    pub fn has_codes_to_continue(&self) -> bool {
        !self.codes_to_continue.is_empty()
    }
}

impl<'a> Segment<'a> {
    fn text(&self) -> &'a str {
        match self {
            Segment::Word(txt, _) | Segment::Term(txt, _) => txt,
        }
    }
    fn pos(&self) -> usize {
        match self {
            Segment::Word(_, pos) | Segment::Term(_, pos) => *pos,
        }
    }
}

impl<'a> AnsiString<'a> {
    pub fn build(c2c: Option<String>, slice: &'a str, len: usize, needs_rst: bool) -> Self {
        Self {
            slice,
            len,
            c2c,
            needs_rst,
        }
    }
}

/// Processes input text with ANSI codes into formatted lines that fit within specified dimensions.
///
/// Takes raw text containing ANSI escape sequences and breaks it into lines that respect
/// both horizontal (character) and vertical (line) constraints while preserving ANSI formatting.
///
/// # Arguments
/// * `input` - Raw input text that may contain ANSI escape sequences
/// * `hspace` - Maximum characters per line (excluding ANSI codes)
/// * `vspace` - Maximum number of lines to generate
/// * `overflow` - Strategy for handling text that exceeds horizontal space
///
/// # Returns
/// Vector of `AnsiString` objects, each representing a formatted line with:
/// - Original text slice (may include ANSI codes)
/// - Continuation codes needed to maintain formatting across line breaks
/// - Actual display length (excluding ANSI codes)
/// - Whether the line needs a reset code for proper formatting
pub fn build_string<'a>(
    input: &'a str,
    hspace: usize,
    vspace: usize,
    overflow: &Overflow,
) -> Vec<AnsiString<'a>> {
    if hspace == 0 {
        return vec![];
    }
    // stack of collected ANSI codes
    let mut queue = CodeQueue::default();
    let mut result = Vec::with_capacity(vspace);
    let mut str_pos = 0;
    let mut end_pos = 0;
    let mut txt_len = 0;

    // Tracks whether the current line needs a reset code to properly close ANSI formatting
    let mut line_reset = false;

    let segments = build_segments(input, overflow);
    let count = segments.len();

    for (i, seg) in segments.iter().enumerate() {
        let is_last_str = i == count - 1;
        let is_init_str = txt_len == 0;
        let is_term_str = matches!(seg, Segment::Term(_, _));

        let (new_codes, txt, total_len, has_rst, needs_rst) = parse_segment(seg, hspace);
        let len = min(total_len, hspace);
        let eol = is_last_str || is_term_str;

        // Separator length: 0 for first segment in line, 1 otherwise.
        let sep_len = (txt_len > 0) as usize;

        if txt_len == 0 {
            str_pos = seg.pos();
            end_pos = str_pos;
            line_reset = queue.has_codes_to_continue();
        }

        // If there is no more space for a segment then wrap-or-truncate the line
        if !is_init_str && txt_len + total_len + sep_len > hspace {
            result.push(AnsiString::build(
                queue.codes_to_continue(),
                &input[str_pos..end_pos],
                txt_len,
                line_reset,
            ));
            // Constituate current segment as initial in the new line
            str_pos = seg.pos();
            end_pos = str_pos + txt.len();
            txt_len = len;
        } else {
            end_pos += txt.len() + sep_len;
            txt_len += len + sep_len;
        }

        // If segment contains reset code at any position wipe out
        // all ANSI codes collected up to the reset code so far...
        if has_rst {
            queue.clear();
            line_reset = needs_rst;
        } else {
            line_reset = line_reset || needs_rst;
        }

        // ...and collect all the codes coming right after the reset code
        queue.collect(new_codes);

        if result.len() < vspace && (txt_len == hspace || eol) {
            result.push(AnsiString::build(
                queue.codes_to_continue(),
                &input[str_pos..end_pos],
                txt_len,
                line_reset,
            ));
            txt_len = 0;
        }

        // Bail out early if there is no more vertical space available
        if result.len() == vspace {
            return result;
        }
    }
    result
}

/// Splits input string into segments for parsing based on overflow strategy.
///
/// - `Overflow::Truncate`: Splits only on newlines, creating one segment per line
/// - `Overflow::WordWrap`: Splits on both newlines and spaces for word-based wrapping
fn build_segments<'a>(input: &'a str, overflow: &Overflow) -> Vec<Segment<'a>> {
    let input_ptr = input.as_ptr();
    match overflow {
        Overflow::Truncate => input
            .lines()
            .map(|s| Segment::Term(s, s.as_ptr() as usize - input_ptr as usize))
            .collect::<Vec<_>>(),
        Overflow::WordWrap => input
            .lines()
            .flat_map(|s| {
                let mut iter = s.split(' ').peekable();

                std::iter::from_fn(move || {
                    iter.next().map(|slice| {
                        let pos = slice.as_ptr() as usize - input_ptr as usize;

                        if iter.peek().is_none() {
                            Segment::Term(slice, pos)
                        } else {
                            Segment::Word(slice, pos)
                        }
                    })
                })
            })
            .collect::<Vec<_>>(),
    }
}

/// Parses a single text segment, extracting ANSI codes and enforcing character limits.
///
/// Returns a tuple containing:
/// - Vector of ANSI escape sequences found in the segment
/// - Text slice (including ANSI codes) truncated to fit the character limit
/// - Actual text length (excluding ANSI codes)  
/// - Whether a reset code was found in the segment
/// - Whether the segment needs a reset code (has unclosed ANSI styling)
fn parse_segment<'a>(segment: &'a Segment, len: usize) -> AnsiSegment<'a> {
    let mut codes = Vec::new();
    let mut expected = AnsiToken::Escape;
    let mut current_code_start = 0;

    let mut txt_len: usize = 0;
    let mut end_pos = None;
    let mut has_rst = false;
    let mut needs_rst = false;
    let mut stop_collecting = false;

    let input = segment.text();

    for (pos, ch) in input.char_indices() {
        match ch {
            '\x1b' if expected == AnsiToken::Escape => {
                expected = AnsiToken::Opening;
                current_code_start = pos;
            }
            '[' if expected == AnsiToken::Opening => expected = AnsiToken::Code,
            'm' if expected == AnsiToken::Code => {
                // Valid SGR sequence terminator
                let sequence = &input[current_code_start..pos + 1];
                let seq_rst = sequence == RST_CODE;

                has_rst = seq_rst;

                if seq_rst {
                    codes.clear();
                } else {
                    codes.push(sequence);
                }
                if !stop_collecting {
                    needs_rst = !has_rst;
                    if end_pos.is_some() {
                        end_pos = Some(pos + 1);
                    }
                }
                expected = AnsiToken::Escape
            }
            '0'..='9' | ';' | ':' if expected == AnsiToken::Code => {
                continue;
            }
            _ if end_pos.is_none() => {
                txt_len += 1;

                if txt_len == len {
                    end_pos = Some(pos + ch.len_utf8());
                }
                expected = AnsiToken::Escape;
            }
            _ => {
                stop_collecting = true;
                expected = AnsiToken::Escape;
                // consume, do nothing
            }
        }
    }
    let slice = &input[0..end_pos.unwrap_or(input.len())];
    (codes, slice, txt_len, has_rst, needs_rst)
}

#[macro_export]
macro_rules! assert_ansi_string {
        ($string:expr, $hspace:expr, $vspace:expr, $overflow:expr, []) => {
            let str = format!($string);
            let segments = $crate::ansi::build_string(&str, $hspace, $vspace, &$overflow);

            assert!(segments.is_empty());
        };
        ($string:expr, $hspace:expr, $vspace:expr, $overflow:expr, [$($segment:tt),*]) => {
            {
                let str = format!($string);
                let segments = $crate::ansi::build_string(&str, $hspace, $vspace, &$overflow);
                let mut segment_index = 0;

                $(
                    assert_ansi_string!(@verify_segment segments[segment_index], $segment);
                    segment_index += 1;
                )+
                assert_eq!(segments.len(), segment_index, "Expected {} segments, found {}", segment_index, segments.len());
            }
        };
        (@verify_segment $seg:expr, { $($field:ident => $value:tt),* }) => {
            let seg = &$seg;
            $(
                assert_ansi_string!(@check_field seg, $field, $value);
            )*
        };
        (@check_field $seg:expr, len, $expected:expr) => {
            assert_eq!($seg.len, $expected);
        };
        (@check_field $seg:expr, txt, $expected:literal) => {
            let formatted = format!($expected);
            assert_eq!($seg.slice.as_ref(), formatted);
        };
        (@check_field $seg:expr, rst, $expected:expr) => {
            assert_eq!($seg.needs_rst, $expected);
        };
    }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_codes_queue_clear() {
        let mut queue = CodeQueue::default();
        queue.collect(vec!["\x1b[31m", "\x1b[1m"]);

        // First call moves collected to continue
        assert_eq!(queue.codes_to_continue(), None);
        assert!(queue.has_codes_to_continue());

        // Clear should mark for reset
        queue.clear();

        // Next call should clear and return existing codes
        assert_eq!(
            queue.codes_to_continue(),
            Some(String::from("\x1b[31m\x1b[1m"))
        );
        assert!(!queue.has_codes_to_continue());
    }

    #[test]
    fn test_codes_queue_multiple_codes_collected() {
        let mut queue = CodeQueue::default();

        // First collected code.
        // Nothing to be applied at the beginning of current line.
        queue.collect(vec!["\x1b[31m"]);
        assert_eq!(queue.codes_to_continue(), None);

        // Second collected code should append to queue of codes to continue
        // but current line should be prepended with previously collected code.
        queue.collect(vec!["\x1b[1m"]);
        assert_eq!(queue.codes_to_continue(), Some(String::from("\x1b[31m")));

        // Finally, next call of `codes_to_continue` should generate a sequence
        // of all codes collected so far.
        assert_eq!(
            queue.codes_to_continue(),
            Some(String::from("\x1b[31m\x1b[1m"))
        );
    }

    #[test]
    fn test_codes_queue_clear_with_new_codes() {
        let mut queue = CodeQueue::default();

        // Set up some continuing codes
        queue.collect(vec!["\x1b[31m", "\x1b[1m"]);
        queue.codes_to_continue();

        // Collect new codes then clear
        queue.collect(vec!["\x1b[32m"]);
        queue.clear();

        // Should get the old continuing codes (before clear) and new codes should be cleared
        assert_eq!(
            queue.codes_to_continue(),
            Some(String::from("\x1b[31m\x1b[1m"))
        );
        assert!(!queue.has_codes_to_continue());
    }

    #[test]
    fn test_codes_queue_empty_states() {
        let mut queue = CodeQueue::default();

        // Empty queue
        assert!(!queue.has_codes_to_continue());
        assert_eq!(queue.codes_to_continue(), None);

        // Empty collection
        queue.collect(vec![]);
        assert_eq!(queue.codes_to_continue(), None);
        assert!(!queue.has_codes_to_continue());

        // Clear empty queue
        queue.clear();
        assert_eq!(queue.codes_to_continue(), None);
        assert!(!queue.has_codes_to_continue());
    }
}
