#[allow(unused)]
mod ansi_string_truncated {
    use fancy_table::ansi::{AnsiString, Overflow, build_string};
    use fancy_table::assert_ansi_string;

    const BLUE: &str = "\x1b[34m";
    const RED: &str = "\x1b[31m";
    const RST: &str = "\x1b[0m";

    const OVERFLOW: Overflow = Overflow::Truncate;

    #[test]
    fn test_edge_bounds() {
        assert_ansi_string!("Hello World", 0, 1, OVERFLOW, []);
        assert_ansi_string!("Hello World", 1, 1, OVERFLOW, [
            {
                txt => "H",
                len => 1,
                rst => false
            }
        ]);
        assert_ansi_string!("Hello World", 30, 1, OVERFLOW, [
            {
                txt => "Hello World",
                len => 11,
                rst => false
            }
        ]);
    }

    #[test]
    fn test_basic_substrings() {
        let input = "Hello World";

        assert_ansi_string!("Hello World", 5, 1, OVERFLOW, [
            {
                txt => "Hello",
                len => 5,
                rst => false
            }
        ]);
        assert_ansi_string!("Hello World", 6, 1, OVERFLOW, [
            {
                txt => "Hello ",
                len => 6,
                rst => false
            }
        ]);
        assert_ansi_string!("Hello World", 11, 1, OVERFLOW, [
            {
                txt => "Hello World",
                len => 11,
                rst => false
            }
        ]);
        assert_ansi_string!("🦀Hello🦀World", 3, 1, OVERFLOW, [
            {
                txt => "🦀He",
                len => 3,
                rst => false
            }
        ]);
        assert_ansi_string!("🦀Hello🦀World", 7, 1, OVERFLOW, [
            {
                txt => "🦀Hello🦀",
                len => 7,
                rst => false
            }
        ]);
        assert_ansi_string!("🦀Hello🦀World", 8, 1, OVERFLOW, [
            {
                txt => "🦀Hello🦀W",
                len => 8,
                rst => false
            }
        ]);
        assert_ansi_string!("🦀Hello🦀World", 12, 1, OVERFLOW, [
            {
                txt => "🦀Hello🦀World",
                len => 12,
                rst => false
            }
        ]);
    }

    #[test]
    fn test_balanced_ansi_codes() {
        assert_ansi_string!("{RED}Hello{RST} World'", 3, 1, OVERFLOW, [
            {
                txt => "{RED}Hel",
                len => 3,
                rst => true
            }
        ]);
        assert_ansi_string!("{RED}Hello{RST}", 5, 1, OVERFLOW, [
            {
                txt => "{RED}Hello{RST}",
                len => 5,
                rst => false
            }
        ]);
        assert_ansi_string!("{RED}Hello{RST} World", 6, 1, OVERFLOW, [
            {
                txt => "{RED}Hello{RST} ",
                len => 6,
                rst => false
            }
        ]);
        assert_ansi_string!("{RED}Hello{RST} World", 11, 1, OVERFLOW, [
            {
                txt => "{RED}Hello{RST} World",
                len => 11,
                rst => false
            }
        ]);
        assert_ansi_string!("{RED}🦀Hello🦀{RST}World", 3, 1, OVERFLOW, [
            {
                txt => "{RED}🦀He",
                len => 3,
                rst => true
            }
        ]);
        assert_ansi_string!("{RED}🦀Hello🦀{RST}World", 7, 1, OVERFLOW, [
            {
                txt => "{RED}🦀Hello🦀{RST}",
                len => 7,
                rst => false
            }
        ]);
        assert_ansi_string!("{RED}🦀Hello🦀{RST}World", 8, 1, OVERFLOW, [
            {
                txt => "{RED}🦀Hello🦀{RST}W",
                len => 8,
                rst => false
            }
        ]);
    }

    #[test]
    fn test_unbalanced_codes() {
        assert_ansi_string!("{RED}🦀Hello🦀{BLUE} World", 3, 1, OVERFLOW, [
            {
                txt => "{RED}🦀He",
                len => 3,
                rst => true
            }
        ]);
        assert_ansi_string!("{RED}🦀Hello🦀{BLUE} World", 7, 1, OVERFLOW, [
            {
                txt => "{RED}🦀Hello🦀{BLUE}",
                len => 7,
                rst => true
            }
        ]);
        assert_ansi_string!("{RED}🦀Hello🦀{BLUE} World", 9, 1, OVERFLOW, [
            {
                txt => "{RED}🦀Hello🦀{BLUE} W",
                len => 9,
                rst => true
            }
        ]);
        assert_ansi_string!("{RED}🦀Hello🦀{BLUE} World", 20, 1, OVERFLOW, [
            {
                txt => "{RED}🦀Hello🦀{BLUE} World",
                len => 13,
                rst => true
            }
        ]);
        assert_ansi_string!("{RED}🦀Hello🦀{RST}{BLUE} World", 20, 1, OVERFLOW, [
            {
                txt => "{RED}🦀Hello🦀{RST}{BLUE} World",
                len => 13,
                rst => true
            }
        ]);
    }

    #[test]
    fn test_unbalanced_codes_with_reset() {
        assert_ansi_string!("{RED}🦀Hello🦀{BLUE}{RST} World", 20, 1, OVERFLOW, [
            {
                txt => "{RED}🦀Hello🦀{BLUE}{RST} World",
                len => 13,
                rst => false
            }
        ]);
        assert_ansi_string!("{RED}🦀Hello🦀{BLUE}{RED} World{RST}", 20, 1, OVERFLOW, [
            {
                txt => "{RED}🦀Hello🦀{BLUE}{RED} World{RST}",
                len => 13,
                rst => false
            }
        ]);
        assert_ansi_string!("{RED}🦀Hello🦀{BLUE} World {RED}{RST}", 20, 1, OVERFLOW, [
            {
                txt => "{RED}🦀Hello🦀{BLUE} World {RED}{RST}",
                len => 14,
                rst => false
            }
        ]);
    }
}

#[allow(unused)]
mod ansi_string_wrapped {
    use fancy_table::ansi::{AnsiString, Overflow, build_string};
    use fancy_table::assert_ansi_string;

    const BLUE: &str = "\x1b[34m";
    const RED: &str = "\x1b[31m";
    const RST: &str = "\x1b[0m";

    const OVERFLOW: Overflow = Overflow::WordWrap;

    #[test]
    fn test_edge_bounds() {
        assert_ansi_string!("Hello World", 0, 1, OVERFLOW, []);
        assert_ansi_string!("Hello World", 1, 1, OVERFLOW, [
            {
                txt => "H",
                len => 1,
                rst => false
            }
        ]);
        assert_ansi_string!("Hello World{BLUE}", 1, 2, OVERFLOW, [
            {
                txt => "H",
                len => 1,
                rst => false
            },
            {
                txt => "W",
                len => 1,
                rst => false
            }
        ]);
        assert_ansi_string!("Hello wo{BLUE}", 1, 2, OVERFLOW, [
            {
                txt => "H",
                len => 1,
                rst => false
            },
            {
                txt => "w",
                len => 1,
                rst => false
            }
        ]);
        assert_ansi_string!("Hello{RED} World{BLUE}", 1, 2, OVERFLOW, [
            {
                txt => "H",
                len => 1,
                rst => false
            },
            {
                txt => "W",
                len => 1,
                rst => true
            }
        ]);
        assert_ansi_string!("Hello World", 30, 1, OVERFLOW, [
            {
                txt => "Hello World",
                len => 11,
                rst => false
            }
        ]);
    }

    #[test]
    fn test_basic_substrings() {
        assert_ansi_string!("Hello World", 3, 2, OVERFLOW, [
            {
                txt => "Hel",
                len => 3,
                rst => false
            },
            {
                txt => "Wor",
                len => 3,
                rst => false
            }
        ]);
        assert_ansi_string!("Hello World", 5, 2, OVERFLOW, [
            {
                txt => "Hello",
                len => 5,
                rst => false
            },
            {
                txt => "World",
                len => 5,
                rst => false
            }
        ]);
        assert_ansi_string!("Hello World", 6, 2, OVERFLOW, [
            {
                txt => "Hello",
                len => 5,
                rst => false
            },
            {
                txt => "World",
                len => 5,
                rst => false
            }
        ]);
        assert_ansi_string!("Hello World", 11, 2, OVERFLOW, [
            {
                txt => "Hello World",
                len => 11,
                rst => false
            }
        ]);
        assert_ansi_string!("Hello Beautiful World", 6, 2, OVERFLOW, [
            {
                txt => "Hello",
                len => 5,
                rst => false
            },
            {
                txt => "Beauti",
                len => 6,
                rst => false
            }
        ]);
        assert_ansi_string!("🦀Hello🦀World", 7, 2, OVERFLOW, [
            {
                txt => "🦀Hello🦀",
                len => 7,
                rst => false
            }
        ]);

        assert_ansi_string!("🦀Hello  🦀World", 3, 2, OVERFLOW, [
            {
                txt => "🦀He",
                len => 3,
                rst => false
            },
            {
                txt => "🦀Wo",
                len => 3,
                rst => false
            }
        ]);
        assert_ansi_string!("🦀Hello 🦀Beautiful 🦀World", 1, 3, OVERFLOW, [
            {
                txt => "🦀",
                len => 1,
                rst => false
            },
            {
                txt => "🦀",
                len => 1,
                rst => false
            },
            {
                txt => "🦀",
                len => 1,
                rst => false
            }
        ]);
        // TODO: for now, let's assume we don't really want leading spaces.
        // They will look weird having justification applied.
        assert_ansi_string!("  🦀H e l l o  🦀World", 8, 2, OVERFLOW, [
            {
                txt => "🦀H e l l",
                len => 8,
                rst => false
            },
            {
                txt => "o ",
                len => 2,
                rst => false
            }

        ]);
    }
}
