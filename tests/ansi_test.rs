mod ansi_string {
    use fancy_table::assert_codes;

    const BLUE: &str = "\x1b[31m";
    const RED: &str = "\x1b[33m";
    const RST: &str = "\x1b[0m";

    #[test]
    fn ansi_strings_single_segment() {
        assert_codes!("Hello", [
            {
                len => 5,
                txt => "Hello",
                sgr => None,
                rst => None
            }
        ]);
        assert_codes!("{RED}Hello", [
            {
                len => 5,
                txt => "Hello",
                sgr => "{RED}",
                rst => None
            }
        ]);
        assert_codes!("Hello{RED}", [
            {
                len => 5,
                txt => "Hello",
                sgr => None,
                rst => None
            }
        ]);
        assert_codes!("{RED}Hello{RST}", [
            {
                len => 5,
                txt => "Hello",
                sgr => "{RED}",
                rst => "{RST}"
            }
        ]);
        assert_codes!("{RED}{RST}Hello", [
            {
                len => 5,
                txt => "Hello",
                sgr => None,
                rst => None
            }
        ]);
        assert_codes!("Hello{RED}{RST}", [
            {
                len => 5,
                txt => "Hello",
                sgr => None,
                rst => None
            }
        ]);
        assert_codes!("Hello{RST}", [
            {
                len => 5,
                txt => "Hello",
                sgr => None,
                rst => "{RST}"
            }
        ]);
        assert_codes!("{RED}{BLUE}Hello", [
            {
                len => 5,
                txt => "Hello",
                sgr => "{RED}{BLUE}",
                rst => None
            }
        ]);
        assert_codes!("{RST}{RED}{BLUE}{RST}{RED}{BLUE}Hello", [
            {
                len => 5,
                txt => "Hello",
                sgr => "{RED}{BLUE}",
                rst => None
            }
        ]);
    }

    #[test]
    fn ansi_strings_single_segment_with_unicode() {
        assert_codes!("🦀Hello🦀", [
            {
                len => 7,
                txt => "🦀Hello🦀",
                sgr => None,
                rst => None
            }
        ]);
        assert_codes!("{RED}🦀Hello🦀", [
            {
                len => 7,
                txt => "🦀Hello🦀",
                sgr => "{RED}",
                rst => None
            }
        ]);
        assert_codes!("🦀Hello🦀{RED}", [
            {
                len => 7,
                txt => "🦀Hello🦀",
                sgr => None,
                rst => None
            }
        ]);
        assert_codes!("{RED}🦀Hello🦀{RST}", [
            {
                len => 7,
                txt => "🦀Hello🦀",
                sgr => "{RED}",
                rst => "{RST}"
            }
        ]);
        assert_codes!("{RED}{RST}🦀Hello🦀", [
            {
                len => 7,
                txt => "🦀Hello🦀",
                sgr => None,
                rst => None
            }
        ]);
        assert_codes!("🦀Hello🦀{RED}{RST}", [
            {
                len => 7,
                txt => "🦀Hello🦀",
                sgr => None,
                rst => None
            }
        ]);
    }

    #[test]
    fn ansi_strings_multi_segment() {
        assert_codes!("{RED}Hello{BLUE}World!", [
            {
                len => 5,
                txt => "Hello",
                sgr => "{RED}",
                rst => None
            },
            {
                len => 6,
                txt => "World!",
                sgr => "{BLUE}",
                rst => None
            }
        ]);
        assert_codes!("{RED}Hello{RST}🦀World🦀", [
            {
                len => 5,
                txt => "Hello",
                sgr => "{RED}",
                rst => "{RST}"
            },
            {
                len => 7,
                txt => "🦀World🦀",
                sgr => None,
                rst => None
            }
        ]);
        assert_codes!("{RED}Hello{RST}{BLUE}🦀World🦀", [
            {
                len => 5,
                txt => "Hello",
                sgr => "{RED}",
                rst => "{RST}"
            },
            {
                len => 7,
                txt => "🦀World🦀",
                sgr => "{BLUE}",
                rst => None
            }
        ]);
    }
}

mod ansi_string_get {
    use fancy_table::AnsiString;

    const RED: &str = "\x1b[33m";
    const BLUE: &str = "\x1b[31m";
    const RST: &str = "\x1b[0m";

    #[test]
    fn get_edge_bounds() {
        let input = "Hello World";
        let ansi_str = AnsiString::new(input);

        assert_eq!(ansi_str.get(0), "");
        assert_eq!(ansi_str.get(30), input);
    }

    #[test]
    fn get_basic_substring() {
        let input = "Hello World";
        let ansi_str = AnsiString::new(input);

        assert_eq!(ansi_str.get(5), "Hello");
        assert_eq!(ansi_str.get(11), "Hello World");
        assert_eq!(ansi_str.get(0), "");
        assert_eq!(ansi_str.get(20), "Hello World");
    }

    #[test]
    fn get_with_ansi_codes() {
        let input = format!("{RED}Hello{RST} World");
        let ansi_str = AnsiString::new(&input);

        assert_eq!(ansi_str.get(5), format!("{RED}Hello{RST}"));
        assert_eq!(ansi_str.get(3), format!("{RED}Hel"));
        assert_eq!(ansi_str.get(11), format!("{RED}Hello{RST} World"));
    }

    #[test]
    fn get_with_unicode() {
        let input = "🦀foo🦀bar";
        let ansi_str = AnsiString::new(input);

        assert_eq!(ansi_str.get(3), "🦀fo");
        assert_eq!(ansi_str.get(5), "🦀foo🦀");
        assert_eq!(ansi_str.get(8), "🦀foo🦀bar");
    }

    #[test]
    fn get_with_unicode_and_ansi() {
        let input = format!("{RED}🦀foo🦀{RST}bar");
        let ansi_str = AnsiString::new(&input);

        assert_eq!(ansi_str.get(3), format!("{RED}🦀fo"));
        assert_eq!(ansi_str.get(5), format!("{RED}🦀foo🦀{RST}"));
        assert_eq!(ansi_str.get(8), format!("{RED}🦀foo🦀{RST}bar"));
    }

    #[test]
    fn get_multiple_codes() {
        let input = format!("{RED}Hello{BLUE} World");
        let ansi_str = AnsiString::new(&input);

        assert_eq!(ansi_str.get(3), format!("{RED}Hel"));
        assert_eq!(ansi_str.get(5), format!("{RED}Hello"));
        assert_eq!(ansi_str.get(8), format!("{RED}Hello{BLUE} Wo"));
        assert_eq!(ansi_str.get(11), format!("{RED}Hello{BLUE} World"));
    }

    #[test]
    fn get_with_owned_sgr() {
        let input = "Hello World";
        let ansi_str = AnsiString::new(input).with_sgr(RED.to_string());

        assert_eq!(ansi_str.get(5), format!("{RED}Hello"));
        assert_eq!(ansi_str.get(11), format!("{RED}Hello World"));
    }
}
