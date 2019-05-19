use std::convert::TryFrom;

pub mod text_window {
    use super::TryFrom;

    #[derive(Eq, PartialEq, Copy, Clone, Debug)]
    pub enum BracketStyle {
        Square,
        Squiggly,
        Circle,
        None,
    }

    impl BracketStyle {
        fn char(&self) -> (char, char) {
            match self {
                BracketStyle::Square => ('[', ']'),
                BracketStyle::Squiggly => ('{', '}'),
                BracketStyle::Circle => ('(', ')'),
                BracketStyle::None => ('\0', '\0'), // null characters
            }
        }
    }

    impl TryFrom<String> for BracketStyle {
        type Error = &'static str;

        fn try_from(value: String) -> Result<Self, Self::Error> {
            match &*value.to_lowercase() {
                "square" => Ok(BracketStyle::Square),
                "squiggly" => Ok(BracketStyle::Squiggly),
                "circle" => Ok(BracketStyle::Circle),
                "none" => Ok(BracketStyle::None),
                _ => Err("does not match any bracket style known")
            }
        }
    }

    fn generate_windows(text: &str, background: char, window_size: u64, bracket_style: BracketStyle)
                        -> Vec<String> {
        /*
        text = test, bg = ' ', window_size = 8
        up: size=5
        [test    ] 0 (0..=last_offset)
        [ test   ] 1
        [  test  ] 2
        [   test ] 3
        [    test] 4
        down: size=3
        [   test ] 3 (1..=last_offset)
        [  test  ] 2
        [ test   ] 1
        */
        let text_width = text.len() as u64;
        let last_offset = window_size - text_width;
        let up = 0..=last_offset; // (1)
        let down = 1..last_offset; // (1, 2) => 2, 1
        let offsets = up.into_iter().chain(down.into_iter().rev());
        offsets
            .map(|n| generate_singular_window(text, background, n, window_size, bracket_style))
            .collect::<Vec<String>>()
    }

    pub fn window_generator(text: &str, background: char, window_size: u64, bracket_style: BracketStyle)
                            -> std::iter::Cycle<std::vec::IntoIter<String>> {
        let frames = generate_windows(text, background, window_size, bracket_style);
        frames.into_iter().cycle()
    }

    fn generate_singular_window(text: &str, background: char, offset: u64, window_size: u64,
                                bracket_style: BracketStyle,
    )
                                -> String {
        fn char_n_times(c: char, n: usize) -> String {
            std::iter::repeat(c)
                .take(n).collect()
        }
        // gsw("test", 0, 5) = [test ] sl_l = 0
        // gsw("test", 1, 5) = [ test] sl_l = 1, offset + len = 5
        // gsw("test", 1, 10) = [ test     ]
        let text_length = text.len() as u64;
        let padding_left = {
            if offset == 0 {
                String::new()
            } else {
                let n = window_size - (window_size - offset);
                char_n_times(background, n as usize)
            }
        };
        let padding_right = {
            if offset + text_length == window_size {
                String::new()
            } else {
                let n = window_size - (offset + text_length);
                char_n_times(background, n as usize)
            }
        };
        let (left_bracket, right_bracket) = bracket_style.char();
        format!("{}{}{}{}{}", left_bracket, padding_left, text, padding_right, right_bracket)
    }

    mod test {
        use super::{BracketStyle, TryFrom,
                    generate_windows, generate_singular_window, window_generator};

        #[test]
        fn bracket_style_parse() {
            let style = BracketStyle::try_from(String::from("square"));
            assert_eq!(style, Ok(BracketStyle::Square));
//            assert!(style.is_ok());
//            if let Ok(parsed_style) = style {
//                assert_eq!(parsed_style, BracketStyle::Square);
//            }
        }

        #[test]
        fn gsw_works() {
            assert_eq!(generate_singular_window("test", ' ',
                                                0, 5, BracketStyle::Square), "[test ]");
            assert_eq!(generate_singular_window("test", ' ',
                                                1, 5, BracketStyle::Squiggly), "{ test}");
            assert_eq!(generate_singular_window("test", ' ', 1, 10, BracketStyle::Circle), "( test     )")
        }

        #[test]
        fn gsw_frames_works() {
            let frames = generate_windows("text", ' ', 5, BracketStyle::Square);
            assert_eq!(frames, vec![
                "[text ]",
                "[ text]",
            ]);
        }

        #[test]
        fn window_generator_works() {
            let gen = window_generator("text", ' ', 5, BracketStyle::Square);
            // take 4 iterations
            let frames = gen.take(4).collect::<Vec<String>>();
            assert_eq!(frames, vec![
                "[text ]",
                "[ text]",
                "[text ]",
                "[ text]"
            ]);

            // lets try a longer one
            let gen = window_generator("abc123", ' ', 9, BracketStyle::Circle);
            // take N iterations
            let frames = gen.take(7).collect::<Vec<String>>();
            assert_eq!(frames, vec![
                "(abc123   )",
                "( abc123  )",
                "(  abc123 )",
                "(   abc123)",
                "(  abc123 )",
                "( abc123  )",
                "(abc123   )"
            ]);
        }
    }
}

pub mod text_window_command {
    use crate::{
        CommandInfo, Command,
    };
    use serenity::{
        model::channel::Message,
        prelude::Context,
    };
    use regex::Regex;
    use super::{
        text_window::BracketStyle,
        TryFrom,
    };
    use std::time::Duration;

    pub struct TextWindow;

    pub struct TextWindowOptions {
        text: String,
        style: BracketStyle,
        background: char,
        window_size: u64,
        delay: u64,
    }

    lazy_static! {
        static ref PATTERN: Regex = {
            Regex::new(".window (?P<delay>\\d+) (?P<window_size>.+) (?P<bracket_style>.+) \'(?P<background>.)\' \"(?P<text>.+)\"")
                .expect("Failed to compile text window regex")
        };
    }

    impl TextWindow {
        pub fn fetch_options<'a>(text: &str) -> Result<TextWindowOptions, failure::Error> {
            let captures = PATTERN.captures(text);
            if let Some(captures) = PATTERN.captures(text) {
                if let (
                    Some(bracket_style),
                    Some(text),
                    Some(background),
                    Some(window_size),
                    Some(delay),
                ) = (
                    captures.name("bracket_style"),
                    captures.name("text"),
                    captures.name("background"),
                    captures.name("window_size"),
                    captures.name("delay"),
                ) {
                    let bracket_style_str = bracket_style.as_str().to_string();
                    if let Ok(style) = BracketStyle::try_from(bracket_style_str) {
                        let text = text.as_str().to_string();
                        let background = background.as_str().to_string().chars().next().unwrap(); // regex guarantee
                        if let (
                            Ok(window_size_parsed),
                            Ok(delay_parsed),
                        ) = (
                            window_size.as_str().parse::<u64>(),
                            delay.as_str().parse::<u64>(),
                        ) {
                            let window_size = window_size_parsed;
                            let delay = delay_parsed;
                            return Ok(TextWindowOptions {
                                text,
                                style,
                                window_size,
                                background,
                                delay
                            });
                        }
                    }
                }
            }
            // this should never happen (matches called before fetch_options)
            Err(failure::err_msg("not a valid match"))
        }
    }

    impl Command for TextWindow {
        fn matches(&self, text: &str) -> bool {
            PATTERN.is_match(text)
        }

        fn run(&self, context: &Context, message: &mut Message) {
            let text_content = message.content_safe();
            if let Ok(options) = TextWindow::fetch_options(&*text_content) {
                let text = options.text;
                let style = options.style;
                let background = options.background;
                let window_size = options.window_size;
                let sleep_delay = std::time::Duration::from_millis(options.delay);
                for line in super::text_window::window_generator(&*text, background, window_size, style) {
                    if let Err(why) = message.edit(|m| m.content(line)) {
                        panic!(why);
                    }
                    std::thread::sleep(sleep_delay);
                }
                println!("done??");
            }
        }

        fn report(&self) -> CommandInfo {
            CommandInfo {
                name: "text_window"
            }
        }
    }
}