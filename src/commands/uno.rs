mod uno {
    #[derive(Eq, PartialEq, Clone, Copy, Debug)]
    enum Color {
        Red,
        Yellow,
        Green,
        Blue,
    }

    impl Color {
        fn all() -> [Color; 4] {
            [Color::Red, Color::Yellow, Color::Green, Color::Blue]
        }
    }

    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    enum DrawSize {
        Two,
        Four,
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    enum Action {
        Draw(DrawSize),
        Skip,
        Reverse,
        SetColor(Option<Box<Action>>), // sets the color & optionally one more action
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    enum Card {
        Numeric(Color, u16),
        Special(Action, Option<Color>),
    }


    fn generate_numeric_color_range(color: Color) -> Vec<Card> {
        (0..=TOTAL_NUMBERED_CARDS).into_iter().chain((1..=TOTAL_NUMBERED_CARDS).into_iter())
            .map(|n| Card::Numeric(color, n as u16)).collect::<Vec<Card>>()
    }

    fn generate_colored_numeric_cards() -> Vec<Card> {
        Color::all().into_iter()
            .map(|&c| generate_numeric_color_range(c))
            .fold(Vec::with_capacity(TOTAL_COLORED_NUMERIC_CARDS), |mut acc, mut next| {
                acc.append(&mut next);
                acc
            })
    }

    fn generate_colored_special_cards(color: Color) -> Vec<Card> {
        vec![
            Card::Special(Action::Skip, Some(color)),
            Card::Special(Action::Reverse, Some(color)),
            Card::Special(Action::Draw(DrawSize::Two), Some(color))
        ]
    }

    fn generate_all_colored_special_cards() -> Vec<Card> {
        std::iter::repeat(
            Color::all().into_iter()
                .map(|&c| generate_colored_special_cards(c))
                .flatten()
                .collect::<Vec<Card>>()
        )
            .take(2)
            .fold(Vec::with_capacity(TOTAL_COLORED_SPECIAL_CARDS), |mut acc, mut next| {
                acc.append(&mut next);
                acc
            })
    }

    fn generate_white_cards() -> Vec<Card> {
        let draw_four = std::iter::repeat(
            Card::Special(
                Action::SetColor(Some(Box::new(
                    Action::Draw(DrawSize::Four)
                ))),
                None,
            )).take(4);
        std::iter::repeat( // set color
                           Card::Special(
                               Action::SetColor(None),
                               None,
                           )
        ).take(4).chain(draw_four).collect::<Vec<Card>>()
    }

    fn generate_special_cards() -> Vec<Card> {
        generate_white_cards().into_iter()
            .chain(generate_all_colored_special_cards()).collect()
    }

    const TOTAL_COLORS: usize = 4;
    const TOTAL_NUMBERED_CARDS: usize = 9;
    const TOTAL_COLORED_NUMERIC_CARDS: usize = TOTAL_NUMBERED_CARDS_FOR_COLOR * TOTAL_COLORS;
    const TOTAL_COLORED_SPECIAL_CARDS: usize = TOTAL_SPECIAL_CARDS_FOR_COLOR * TOTAL_COLORS * 2;
    const TOTAL_NUMBERED_CARDS_FOR_COLOR: usize = 19;
    const TOTAL_WHITE_CARDS: usize = 4 * 2;
    const TOTAL_SPECIAL_CARDS: usize = TOTAL_COLORED_SPECIAL_CARDS + TOTAL_WHITE_CARDS;
    const TOTAL_CARDS_IN_FULL_DECK: usize = 108;
    const TOTAL_SPECIAL_CARDS_FOR_COLOR: usize = 3;


    fn generate_deck() -> Vec<Card> {
        generate_colored_numeric_cards().into_iter()
            .chain(generate_special_cards().into_iter()).collect()
    }

    mod test {
        use super::*;

        #[test]
        fn constants_are_correct() {
            assert_eq!(TOTAL_CARDS_IN_FULL_DECK,
                       TOTAL_SPECIAL_CARDS + TOTAL_COLORED_NUMERIC_CARDS
            );
        }

        #[test]
        fn generate_numeric_color_range_works() {
            let blues = generate_numeric_color_range(Color::Blue);
            assert_eq!(blues.len(), TOTAL_NUMBERED_CARDS_FOR_COLOR);
        }

        #[test]
        fn generate_white_cards_works() {
            let white_cards = generate_white_cards();
            assert_eq!(white_cards.len(), TOTAL_WHITE_CARDS);
        }

        #[test]
        fn generate_colored_special_cards_works() {
            let blue_special_cards = generate_colored_special_cards(Color::Blue);
            assert_eq!(blue_special_cards.len(), TOTAL_SPECIAL_CARDS_FOR_COLOR);
        }

        #[test]
        fn generate_all_colored_special_cards_works() {
            let colored_special_cards = generate_all_colored_special_cards();
            assert_eq!(colored_special_cards.len(), TOTAL_COLORED_SPECIAL_CARDS)
        }

        #[test]
        fn generate_special_cards_works() {
            let special_cards = generate_special_cards();
            assert_eq!(special_cards.len(), TOTAL_SPECIAL_CARDS);
        }

        #[test]
        fn generate_entire_deck_works() {
            let deck = generate_deck();
            assert_eq!(deck.len(), TOTAL_CARDS_IN_FULL_DECK);
        }
    }
}
