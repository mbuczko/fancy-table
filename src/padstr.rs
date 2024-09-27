use std::collections::VecDeque;

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
    inner: VecDeque<Chunk<'a>>,
}

fn should_wrap(agg: &str, s: &Chunk, hspace: usize) -> bool {
    let line_start = agg.is_empty();
    !line_start && (agg.len() + (!line_start as usize) + s.inner().len() > hspace)
}

fn center_string(s: &str, width: usize) -> String {
    let padding = width - s.len();
    let left_padding = padding / 2;
    let right_padding = padding - left_padding;

    format!("{:>left_padding$}{s}{:>right_padding$}", "", "")
}

fn rightpad_string(s: &str, width: usize) -> String {
    format!("{:width$}", s)
}

fn leftpad_string(s: &str, width: usize) -> String {
    format!("{:>width$}", s)
}

impl<'a> Chunk<'a> {
    fn inner(&self) -> &'a str {
        match self {
            Self::Word(s) | Self::Term(s) => s,
        }
    }
}

impl<'a> PadStr<'a> {
    pub fn truncating(s: &'a str) -> Self {
        let inner = s.lines().map(Chunk::Term).collect::<VecDeque<_>>();
        Self { inner }
    }

    pub fn wrapping(s: &'a str) -> Self {
        let inner = s
            .lines()
            .flat_map(|l| {
                let sp = l.split_whitespace();
                let count = sp.clone().count();
                sp.enumerate().map(move |(i, w)| {
                    if i == count - 1 {
                        Chunk::Term(w)
                    } else {
                        Chunk::Word(w)
                    }
                })
            })
            .collect::<VecDeque<_>>();

        Self { inner }
    }

    pub fn paddify(&self, hspace: usize, vspace: usize, pad: Pad) -> VecDeque<String> {
        let mut bag = VecDeque::new();
        let mut agg = String::default();

        for (i, s) in self.inner.iter().enumerate() {
            let last_str = i == self.inner.len() - 1;
            let term_str = matches!(s, Chunk::Term(_));

            if !should_wrap(&agg, s, hspace) {
                if !agg.is_empty() {
                    agg.push(' ');
                }
                agg.push_str(s.inner());
            } else {
                bag.push_back(self.pad_str(&agg, hspace, &pad));
                agg = s.inner().to_owned();
            }
            if bag.len() < vspace && (agg.len() == hspace || last_str || term_str) {
                bag.push_back(self.pad_str(&agg, hspace, &pad));
                agg = String::default();
            }
            if bag.len() == vspace {
                return bag;
            }
        }
        bag
    }

    fn pad_str(&self, s: &str, hspace: usize, just: &Pad) -> String {
        let subs = s.get(0..hspace).unwrap_or(s);
        match just {
            Pad::Left => leftpad_string(subs, hspace),
            Pad::Right => rightpad_string(subs, hspace),
            Pad::Center => center_string(subs, hspace),
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
