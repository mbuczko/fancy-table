#[allow(unused)]
mod ansi_string {
    use fancy_table::AnsiString;
    use fancy_table::assert_segments;

    use crate::ansi_string;

    const BLUE: &str = "\x1b[34m";
    const RED: &str = "\x1b[31m";
    const RST: &str = "\x1b[0m";

    #[test]
    fn ansi_string_no_text() {
        let input = RED.to_string();
        let ansi_string = AnsiString::new(&input);

        assert!(ansi_string.is_empty());
    }

    #[test]
    fn malformed_codes() {
        assert_segments!("\x1b31mHello", [
            {
                len => 9,
                txt => "\x1b31mHello",
                sgr => None,
                rst => None
            }
        ]);
        assert_segments!("\x1b[31Hello", [
            {
                len => 9,
                txt => "\x1b[31Hello",
                sgr => None,
                rst => None
            }
        ]);
        assert_segments!("{RED}Hello\x1b[0", [
            {
                len => 8,
                txt => "Hello\x1b[0",
                sgr => "{RED}",
                rst => None
            }
        ]);
        assert_segments!("{RED}Hello[0m", [
            {
                len => 8,
                txt => "Hello[0m",
                sgr => "{RED}",
                rst => None
            }
        ])
    }

    #[test]
    fn ansi_strings_single_segment() {
        assert_segments!("Hello", [
            {
                len => 5,
                txt => "Hello",
                sgr => None,
                rst => None
            }
        ]);
        assert_segments!("{RED}Hello", [
            {
                len => 5,
                txt => "Hello",
                sgr => "{RED}",
                rst => None
            }
        ]);
        assert_segments!("Hello{RED}", [
            {
                len => 5,
                txt => "Hello",
                sgr => None,
                rst => None
            }
        ]);
        assert_segments!("{RED}Hello{RST}", [
            {
                len => 5,
                txt => "Hello",
                sgr => "{RED}",
                rst => "{RST}"
            }
        ]);
        assert_segments!("{RED}{RST}Hello", [
            {
                len => 5,
                txt => "Hello",
                sgr => None,
                rst => None
            }
        ]);
        assert_segments!("Hello{RED}{RST}", [
            {
                len => 5,
                txt => "Hello",
                sgr => None,
                rst => None
            }
        ]);
        assert_segments!("Hello{RST}", [
            {
                len => 5,
                txt => "Hello",
                sgr => None,
                rst => "{RST}"
            }
        ]);
        assert_segments!("{RED}{BLUE}Hello", [
            {
                len => 5,
                txt => "Hello",
                sgr => "{RED}{BLUE}",
                rst => None
            }
        ]);
        assert_segments!("{RST}{RED}{BLUE}{RST}{RED}{BLUE}Hello", [
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
        assert_segments!("🦀Hello🦀", [
            {
                len => 7,
                txt => "🦀Hello🦀",
                sgr => None,
                rst => None
            }
        ]);
        assert_segments!("{RED}🦀Hello🦀", [
            {
                len => 7,
                txt => "🦀Hello🦀",
                sgr => "{RED}",
                rst => None
            }
        ]);
        assert_segments!("🦀Hello🦀{RED}", [
            {
                len => 7,
                txt => "🦀Hello🦀",
                sgr => None,
                rst => None
            }
        ]);
        assert_segments!("{RED}🦀Hello🦀{RST}", [
            {
                len => 7,
                txt => "🦀Hello🦀",
                sgr => "{RED}",
                rst => "{RST}"
            }
        ]);
        assert_segments!("{RED}{RST}🦀Hello🦀", [
            {
                len => 7,
                txt => "🦀Hello🦀",
                sgr => None,
                rst => None
            }
        ]);
        assert_segments!("🦀Hello🦀{RED}{RST}", [
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
        assert_segments!("{RED}Hello{BLUE}World!", [
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
        assert_segments!("{RED}Hello{RST}🦀World🦀", [
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
        assert_segments!("{RED}Hello{RST}{BLUE}🦀World🦀", [
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

    const BLUE: &str = "\x1b[34m";
    const RED: &str = "\x1b[31m";
    const RST: &str = "\x1b[0m";

    #[test]
    fn get_edge_bounds() {
        let input = "Hello World";
        let ansi_str = AnsiString::new(input);

        assert_eq!(ansi_str.get(0).tupled(), ("", 0));
        assert_eq!(ansi_str.get(30).tupled(), (input, 11));
    }

    #[test]
    fn get_basic_substring() {
        let input = "Hello World";
        let ansi_str = AnsiString::new(input);

        assert_eq!(ansi_str.get(0).tupled(), ("", 0));
        assert_eq!(ansi_str.get(5).tupled(), ("Hello", 5));
        assert_eq!(ansi_str.get(11).tupled(), ("Hello World", 11));
        assert_eq!(ansi_str.get(20).tupled(), ("Hello World", 11));
    }

    #[test]
    fn get_with_ansi_codes() {
        let input = format!("{RED}Hello{RST} World");
        let ansi_str = AnsiString::new(&input);

        assert_eq!(ansi_str.get(3).tupled(), (format!("{RED}Hel").as_str(), 3));
        assert_eq!(
            ansi_str.get(5).tupled(),
            (format!("{RED}Hello{RST}").as_str(), 5)
        );
        assert_eq!(
            ansi_str.get(11).tupled(),
            (format!("{RED}Hello{RST} World").as_str(), 11)
        );
    }

    #[test]
    fn get_with_unicode() {
        let input = "🦀foo🦀bar";
        let ansi_str = AnsiString::new(input);

        assert_eq!(ansi_str.get(3).tupled(), ("🦀fo", 3));
        assert_eq!(ansi_str.get(5).tupled(), ("🦀foo🦀", 5));
        assert_eq!(ansi_str.get(8).tupled(), ("🦀foo🦀bar", 8));
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
        let ansi_str = AnsiString::new(input).with_sgr(Some(RED.to_string()));

        assert_eq!(ansi_str.get(5), format!("{RED}Hello"));
        assert_eq!(ansi_str.get(11), format!("{RED}Hello World"));
    }

    #[test]
    fn get_with_appended_strings() {
        let str = "Hello  World";
        let splits = str.split(' ').collect::<Vec<_>>();
        let mut input_1 = AnsiString::new(splits[0]);
        let input_2 = AnsiString::new(splits[1]);
        let input_3 = AnsiString::new(splits[2]);

        input_1.append(input_2);
        input_1.append(input_3);
        assert_eq!(input_1.get(12).tupled(), (str, 12));
    }

    #[test]
    fn get_terminated_reset_with_no_ansi() {
        let ansi_str = AnsiString::new("Hello World");

        let slice = ansi_str.get(5);
        assert!(!slice.needs_rst);

        let slice = ansi_str.get(4);
        assert!(!slice.needs_rst);
    }

    #[test]
    fn get_terminated_reset_with_ansi() {
        let input = format!("{RED}Hello{RST} World");
        let ansi_str = AnsiString::new(&input);

        let slice = ansi_str.get(5);
        assert!(!slice.needs_rst);

        let slice = ansi_str.get(4);
        assert!(slice.needs_rst);
    }
}
