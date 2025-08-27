#[cfg(test)]
mod test_wrapping {
    use fancy_table::JustedString;
    use fancy_table::Justify;

    #[test]
    fn justify_center_fit_hspace() {
        assert_eq!(
            JustedString::wrapping("Ala ma kota").justify(6, 2, Justify::Center),
            vec!["Ala ma", " kota "]
        );
    }

    #[test]
    fn justify_center_enough_hspace() {
        assert_eq!(
            JustedString::wrapping("Ala ma kota").justify(10, 2, Justify::Center),
            vec!["  Ala ma  ", "   kota   "]
        );
    }

    #[test]
    fn justify_center_no_enough_vspace() {
        assert_eq!(
            JustedString::wrapping("Ala ma kota").justify(8, 1, Justify::Center),
            vec![" Ala ma "]
        )
    }

    #[test]
    fn justify_center_minimal_hspace() {
        assert_eq!(
            JustedString::wrapping("Ala ma kota").justify(1, 3, Justify::Center),
            vec!["A", "m", "k"]
        );
    }

    #[test]
    fn justify_center_no_enough_hspace() {
        assert_eq!(
            JustedString::wrapping("Ala ma kota").justify(2, 3, Justify::Center),
            vec!["Al", "ma", "ko"]
        );
    }

    #[test]
    fn justify_center_no_enough_hspace_and_vspace() {
        assert_eq!(
            JustedString::wrapping("Ala ma kota").justify(1, 1, Justify::Center),
            vec!["A"]
        );
    }

    #[test]
    fn justify_left_enough_hspace() {
        assert_eq!(
            JustedString::wrapping("Ala ma kota").justify(10, 2, Justify::Left),
            vec!["Ala ma    ", "kota      "]
        );
    }

    #[test]
    fn justify_right_enough_hspace() {
        assert_eq!(
            JustedString::wrapping("Ala ma kota").justify(10, 2, Justify::Right),
            vec!["    Ala ma", "      kota"]
        );
    }

    #[test]
    fn justify_right_not_enough_hspace() {
        assert_eq!(
            JustedString::wrapping("Ala ma kota").justify(5, 2, Justify::Right),
            vec!["  Ala", "   ma"]
        );
    }

    #[test]
    fn justify_line_with_wrapping() {
        assert_eq!(
            JustedString::wrapping("Ala ma kota\nA kot ma Alę").justify(7, 5, Justify::Left),
            vec!["Ala ma ", "kota   ", "A kot  ", "ma Alę "]
        );
    }

    #[test]
    fn justify_line_with_enough_hspace() {
        assert_eq!(
            JustedString::truncating("Ala ma kota\nA kot ma Alę").justify(15, 2, Justify::Right),
            vec!["    Ala ma kota", "   A kot ma Alę"]
        );
    }

    #[test]
    fn justify_line_with_not_enough_hspace() {
        assert_eq!(
            JustedString::truncating("Ala ma kota\nA kot ma Alę").justify(8, 2, Justify::Left),
            vec!["Ala ma k", "A kot ma"]
        );
    }

    #[test]
    fn justify_line_with_not_enough_hspace_and_vspace() {
        assert_eq!(
            JustedString::truncating("Ala ma kota\nA kot ma Alę\nOna go kocha\nA on ją wcale")
                .justify(8, 3, Justify::Center),
            vec!["Ala ma k", "A kot ma", "Ona go k"]
        );
    }
}

#[cfg(test)]
mod test_wrapping_with_ansi_codes {
    use fancy_table::JustedString;
    use fancy_table::Justify;

    #[test]
    fn justify_with_ansi_codes() {
        assert_eq!(
            JustedString::wrapping("Ala ma\x1b[31m kota").justify(1, 2, Justify::Center),
            vec!["A", "m"]
        );
        assert_eq!(
            JustedString::wrapping("Ala ma \x1b[31mkota a kot ma Alę").justify(
                6,
                2,
                Justify::Center
            ),
            vec!["Ala ma", "\x1b[31mkota a\x1b[0m"]
        );
        assert_eq!(
            JustedString::wrapping("Ala ma\x1b[31m kota").justify(11, 2, Justify::Center),
            vec!["Ala ma\x1b[31m kota\x1b[0m"]
        );
        assert_eq!(
            JustedString::wrapping("Ala ma \x1b[33mkota \"\x1b[0m").justify(12, 1, Justify::Center),
            vec!["Ala ma \x1b[33mkota \x1b[0m"]
        );
        assert_eq!(
            JustedString::wrapping("\x1b[31mAla\x1b[0m ma \x1b[31mkota\x1b[0m").justify(
                6,
                2,
                Justify::Center
            ),
            vec!["\x1b[31mAla\x1b[0m ma", " \x1b[31mkota\x1b[0m "]
        );
        assert_eq!(
            JustedString::wrapping("\x1b[31mAla\nkot\x1b[0m").justify(4, 5, Justify::Left),
            vec!["\x1b[31mAla \x1b[0m", "\x1b[31mkot\x1b[0m ",]
        );
        assert_eq!(
            JustedString::wrapping("\x1b[31mAla\nkot").justify(4, 5, Justify::Left),
            vec!["\x1b[31mAla \x1b[0m", "\x1b[31mkot \x1b[0m",]
        );
        assert_eq!(
            JustedString::wrapping("\x1b[31mAla ma kota\nA kot\x1b[0m ma Alę").justify(
                7,
                5,
                Justify::Left
            ),
            vec![
                "\x1b[31mAla ma \x1b[0m",
                "\x1b[31mkota   \x1b[0m",
                "\x1b[31mA kot\x1b[0m  ",
                "ma Alę "
            ]
        );
    }
}
