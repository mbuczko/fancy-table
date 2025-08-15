#[cfg(test)]
mod test {
    use fancy_table::JustedString;
    use fancy_table::Justify;

    #[test]
    fn justify_center_fit_hspace() {
        let js = JustedString::wrapping("Ala ma kota");
        let lines = js.justify(6, 2, Justify::Center);
        assert_eq!(lines, vec!["Ala ma", " kota "]);
    }

    #[test]
    fn justify_center_enough_hspace() {
        let js = JustedString::wrapping("Ala ma kota");
        let lines = js.justify(10, 2, Justify::Center);
        assert_eq!(lines, vec!["  Ala ma  ", "   kota   "]);
    }

    #[test]
    fn justify_center_no_enough_vspace() {
        let js = JustedString::wrapping("Ala ma kota");
        let lines = js.justify(8, 1, Justify::Center);
        assert_eq!(lines, vec![" Ala ma "])
    }

    #[test]
    fn justify_center_no_enough_hspace() {
        let js = JustedString::wrapping("Ala ma kota");
        let lines = js.justify(2, 3, Justify::Center);
        assert_eq!(lines, vec!["Al", "ma", "ko"]);
    }

    #[test]
    fn justify_center_no_enough_hspace_and_vspace() {
        let js = JustedString::wrapping("Ala ma kota");
        let lines = js.justify(1, 2, Justify::Center);
        assert_eq!(lines, vec!["A", "m"]);
    }

    #[test]
    fn justify_left_enough_hspace() {
        let js = JustedString::wrapping("Ala ma kota");
        let lines = js.justify(10, 2, Justify::Left);
        assert_eq!(lines, vec!["Ala ma    ", "kota      "]);
    }

    #[test]
    fn justify_right_enough_hspace() {
        let js = JustedString::wrapping("Ala ma kota");
        let lines = js.justify(10, 2, Justify::Right);
        assert_eq!(lines, vec!["    Ala ma", "      kota"]);
    }

    #[test]
    fn justify_right_not_enough_hspace() {
        let js = JustedString::wrapping("Ala ma kota");
        let lines = js.justify(5, 2, Justify::Right);
        assert_eq!(lines, vec!["  Ala", "   ma"]);
    }

    #[test]
    fn justify_line_with_wrapping() {
        let js = JustedString::wrapping("Ala ma kota\nA kot ma Alę");
        let lines = js.justify(7, 5, Justify::Left);
        assert_eq!(lines, vec!["Ala ma ", "kota   ", "A kot  ", "ma Alę "]);
    }

    #[test]
    fn justify_line_with_enough_hspace() {
        let js = JustedString::truncating("Ala ma kota\nA kot ma Alę");
        let lines = js.justify(15, 2, Justify::Right);
        assert_eq!(lines, vec!["    Ala ma kota", "   A kot ma Alę"]);
    }

    #[test]
    fn justify_line_with_not_enough_hspace() {
        let js = JustedString::truncating("Ala ma kota\nA kot ma Alę");
        let lines = js.justify(8, 2, Justify::Left);
        assert_eq!(lines, vec!["Ala ma k", "A kot ma"]);
    }

    #[test]
    fn justify_line_with_not_enough_hspace_and_vspace() {
        let js = JustedString::truncating("Ala ma kota\nA kot ma Alę\nOna go kocha\nA on ją wcale");
        let lines = js.justify(8, 3, Justify::Center);
        assert_eq!(lines, vec!["Ala ma k", "A kot ma", "Ona go k"]);
    }

    #[test]
    fn justify_with_ansi_codes() {
        let js = JustedString::wrapping("\x1b[31mAla\x1b[0m ma \x1b[31mkota\x1b[0m");
        let lines = js.justify(6, 2, Justify::Center);
        assert_eq!(
            lines,
            vec!["\x1b[31mAla\x1b[0m ma", " \x1b[31mkota\x1b[0m "]
        );
    }
    #[test]
    fn justify_line_with_wrapping_with_newlines_and_ansi_codes() {
        let js = JustedString::wrapping("\x1b[31mAla\nkot\x1b[0m");
        let lines = js.justify(4, 5, Justify::Left);
        assert_eq!(lines, vec!["\x1b[31mAla \x1b[0m", "\x1b[31mkot\x1b[0m ",]);
    }

    #[test]
    fn justify_line_with_wrapping_with_ansi_codes() {
        let js = JustedString::wrapping("\x1b[31mAla ma kota\nA kot\x1b[0m ma Alę");
        let lines = js.justify(7, 5, Justify::Left);
        assert_eq!(
            lines,
            vec![
                "\x1b[31mAla ma \x1b[0m",
                "\x1b[31mkota   \x1b[0m",
                "\x1b[31mA kot\x1b[0m  ",
                "ma Alę "
            ]
        );
    }
}
