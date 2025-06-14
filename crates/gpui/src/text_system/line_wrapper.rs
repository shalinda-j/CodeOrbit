﻿use crate::{FontId, FontRun, Pixels, PlatformTextSystem, SharedString, TextRun, px};
use collections::HashMap;
use std::{iter, sync::Arc};

/// The GPUI line wrapper, used to wrap lines of text to a given width.
pub struct LineWrapper {
    platform_text_system: Arc<dyn PlatformTextSystem>,
    pub(crate) font_id: FontId,
    pub(crate) font_size: Pixels,
    cached_ascii_char_widths: [Option<Pixels>; 128],
    cached_other_char_widths: HashMap<char, Pixels>,
}

impl LineWrapper {
    /// The maximum indent that can be applied to a line.
    pub const MAX_INDENT: u32 = 256;

    pub(crate) fn new(
        font_id: FontId,
        font_size: Pixels,
        text_system: Arc<dyn PlatformTextSystem>,
    ) -> Self {
        Self {
            platform_text_system: text_system,
            font_id,
            font_size,
            cached_ascii_char_widths: [None; 128],
            cached_other_char_widths: HashMap::default(),
        }
    }

    /// Wrap a line of text to the given width with this wrapper's font and font size.
    pub fn wrap_line<'a>(
        &'a mut self,
        fragments: &'a [LineFragment],
        wrap_width: Pixels,
    ) -> impl Iterator<Item = Boundary> + 'a {
        let mut width = px(0.);
        let mut first_non_whitespace_ix = None;
        let mut indent = None;
        let mut last_candidate_ix = 0;
        let mut last_candidate_width = px(0.);
        let mut last_wrap_ix = 0;
        let mut prev_c = '\0';
        let mut index = 0;
        let mut candidates = fragments
            .into_iter()
            .flat_map(move |fragment| fragment.wrap_boundary_candidates())
            .peekable();
        iter::from_fn(move || {
            for candidate in candidates.by_ref() {
                let ix = index;
                index += candidate.len_utf8();
                let mut new_prev_c = prev_c;
                let item_width = match candidate {
                    WrapBoundaryCandidate::Char { character: c } => {
                        if c == '\n' {
                            continue;
                        }

                        if Self::is_word_char(c) {
                            if prev_c == ' ' && c != ' ' && first_non_whitespace_ix.is_some() {
                                last_candidate_ix = ix;
                                last_candidate_width = width;
                            }
                        } else {
                            // CJK may not be space separated, e.g.: `Hello worldä½ å¥½ä¸–ç•Œ`
                            if c != ' ' && first_non_whitespace_ix.is_some() {
                                last_candidate_ix = ix;
                                last_candidate_width = width;
                            }
                        }

                        if c != ' ' && first_non_whitespace_ix.is_none() {
                            first_non_whitespace_ix = Some(ix);
                        }

                        new_prev_c = c;

                        self.width_for_char(c)
                    }
                    WrapBoundaryCandidate::Element {
                        width: element_width,
                        ..
                    } => {
                        if prev_c == ' ' && first_non_whitespace_ix.is_some() {
                            last_candidate_ix = ix;
                            last_candidate_width = width;
                        }

                        if first_non_whitespace_ix.is_none() {
                            first_non_whitespace_ix = Some(ix);
                        }

                        element_width
                    }
                };

                width += item_width;
                if width > wrap_width && ix > last_wrap_ix {
                    if let (None, Some(first_non_whitespace_ix)) = (indent, first_non_whitespace_ix)
                    {
                        indent = Some(
                            Self::MAX_INDENT.min((first_non_whitespace_ix - last_wrap_ix) as u32),
                        );
                    }

                    if last_candidate_ix > 0 {
                        last_wrap_ix = last_candidate_ix;
                        width -= last_candidate_width;
                        last_candidate_ix = 0;
                    } else {
                        last_wrap_ix = ix;
                        width = item_width;
                    }

                    if let Some(indent) = indent {
                        width += self.width_for_char(' ') * indent as f32;
                    }

                    return Some(Boundary::new(last_wrap_ix, indent.unwrap_or(0)));
                }

                prev_c = new_prev_c;
            }

            None
        })
    }

    /// Truncate a line of text to the given width with this wrapper's font and font size.
    pub fn truncate_line(
        &mut self,
        line: SharedString,
        truncate_width: Pixels,
        truncation_suffix: &str,
        runs: &mut Vec<TextRun>,
    ) -> SharedString {
        let mut width = px(0.);
        let mut suffix_width = truncation_suffix
            .chars()
            .map(|c| self.width_for_char(c))
            .fold(px(0.0), |a, x| a + x);
        let mut char_indices = line.char_indices();
        let mut truncate_ix = 0;
        for (ix, c) in char_indices {
            if width + suffix_width < truncate_width {
                truncate_ix = ix;
            }

            let char_width = self.width_for_char(c);
            width += char_width;

            if width.floor() > truncate_width {
                let result =
                    SharedString::from(format!("{}{}", &line[..truncate_ix], truncation_suffix));
                update_runs_after_truncation(&result, truncation_suffix, runs);

                return result;
            }
        }

        line
    }

    pub(crate) fn is_word_char(c: char) -> bool {
        // ASCII alphanumeric characters, for English, numbers: `Hello123`, etc.
        c.is_ascii_alphanumeric() ||
        // Latin script in Unicode for French, German, Spanish, etc.
        // Latin-1 Supplement
        // https://en.wikipedia.org/wiki/Latin-1_Supplement
        matches!(c, '\u{00C0}'..='\u{00FF}') ||
        // Latin Extended-A
        // https://en.wikipedia.org/wiki/Latin_Extended-A
        matches!(c, '\u{0100}'..='\u{017F}') ||
        // Latin Extended-B
        // https://en.wikipedia.org/wiki/Latin_Extended-B
        matches!(c, '\u{0180}'..='\u{024F}') ||
        // Cyrillic for Russian, Ukrainian, etc.
        // https://en.wikipedia.org/wiki/Cyrillic_script_in_Unicode
        matches!(c, '\u{0400}'..='\u{04FF}') ||
        // Some other known special characters that should be treated as word characters,
        // e.g. `a-b`, `var_name`, `I'm`, '@mention`, `#hashtag`, `100%`, `3.1415`, `2^3`, `a~b`, etc.
        matches!(c, '-' | '_' | '.' | '\'' | '$' | '%' | '@' | '#' | '^' | '~' | ',') ||
        // Characters that used in URL, e.g. `https://github.com/CodeOrbit-industries/CodeOrbit?a=1&b=2` for better wrapping a long URL.
        matches!(c,  '/' | ':' | '?' | '&' | '=') ||
        // `â‹¯` character is special used in CodeOrbit, to keep this at the end of the line.
        matches!(c, 'â‹¯')
    }

    #[inline(always)]
    fn width_for_char(&mut self, c: char) -> Pixels {
        if (c as u32) < 128 {
            if let Some(cached_width) = self.cached_ascii_char_widths[c as usize] {
                cached_width
            } else {
                let width = self.compute_width_for_char(c);
                self.cached_ascii_char_widths[c as usize] = Some(width);
                width
            }
        } else if let Some(cached_width) = self.cached_other_char_widths.get(&c) {
            *cached_width
        } else {
            let width = self.compute_width_for_char(c);
            self.cached_other_char_widths.insert(c, width);
            width
        }
    }

    fn compute_width_for_char(&self, c: char) -> Pixels {
        let mut buffer = [0; 4];
        let buffer = c.encode_utf8(&mut buffer);
        self.platform_text_system
            .layout_line(
                buffer,
                self.font_size,
                &[FontRun {
                    len: buffer.len(),
                    font_id: self.font_id,
                }],
            )
            .width
    }
}

fn update_runs_after_truncation(result: &str, ellipsis: &str, runs: &mut Vec<TextRun>) {
    let mut truncate_at = result.len() - ellipsis.len();
    let mut run_end = None;
    for (run_index, run) in runs.iter_mut().enumerate() {
        if run.len <= truncate_at {
            truncate_at -= run.len;
        } else {
            run.len = truncate_at + ellipsis.len();
            run_end = Some(run_index + 1);
            break;
        }
    }
    if let Some(run_end) = run_end {
        runs.truncate(run_end);
    }
}

/// A fragment of a line that can be wrapped.
pub enum LineFragment<'a> {
    /// A text fragment consisting of characters.
    Text {
        /// The text content of the fragment.
        text: &'a str,
    },
    /// A non-text element with a fixed width.
    Element {
        /// The width of the element in pixels.
        width: Pixels,
        /// The UTF-8 encoded length of the element.
        len_utf8: usize,
    },
}

impl<'a> LineFragment<'a> {
    /// Creates a new text fragment from the given text.
    pub fn text(text: &'a str) -> Self {
        LineFragment::Text { text }
    }

    /// Creates a new non-text element with the given width and UTF-8 encoded length.
    pub fn element(width: Pixels, len_utf8: usize) -> Self {
        LineFragment::Element { width, len_utf8 }
    }

    fn wrap_boundary_candidates(&self) -> impl Iterator<Item = WrapBoundaryCandidate> {
        let text = match self {
            LineFragment::Text { text } => text,
            LineFragment::Element { .. } => "\0",
        };
        text.chars().map(move |character| {
            if let LineFragment::Element { width, len_utf8 } = self {
                WrapBoundaryCandidate::Element {
                    width: *width,
                    len_utf8: *len_utf8,
                }
            } else {
                WrapBoundaryCandidate::Char { character }
            }
        })
    }
}

enum WrapBoundaryCandidate {
    Char { character: char },
    Element { width: Pixels, len_utf8: usize },
}

impl WrapBoundaryCandidate {
    pub fn len_utf8(&self) -> usize {
        match self {
            WrapBoundaryCandidate::Char { character } => character.len_utf8(),
            WrapBoundaryCandidate::Element { len_utf8: len, .. } => *len,
        }
    }
}

/// A boundary between two lines of text.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Boundary {
    /// The index of the last character in a line
    pub ix: usize,
    /// The indent of the next line.
    pub next_indent: u32,
}

impl Boundary {
    fn new(ix: usize, next_indent: u32) -> Self {
        Self { ix, next_indent }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Font, FontFeatures, FontStyle, FontWeight, Hsla, TestAppContext, TestDispatcher, font,
    };
    #[cfg(target_os = "macos")]
    use crate::{TextRun, WindowTextSystem, WrapBoundary};
    use rand::prelude::*;

    fn build_wrapper() -> LineWrapper {
        let dispatcher = TestDispatcher::new(StdRng::seed_from_u64(0));
        let cx = TestAppContext::build(dispatcher, None);
        let id = cx.text_system().font_id(&font("CodeOrbit Plex Mono")).unwrap();
        LineWrapper::new(id, px(16.), cx.text_system().platform_text_system.clone())
    }

    fn generate_test_runs(input_run_len: &[usize]) -> Vec<TextRun> {
        input_run_len
            .iter()
            .map(|run_len| TextRun {
                len: *run_len,
                font: Font {
                    family: "Dummy".into(),
                    features: FontFeatures::default(),
                    fallbacks: None,
                    weight: FontWeight::default(),
                    style: FontStyle::Normal,
                },
                color: Hsla::default(),
                background_color: None,
                underline: None,
                strikethrough: None,
            })
            .collect()
    }

    #[test]
    fn test_wrap_line() {
        let mut wrapper = build_wrapper();

        assert_eq!(
            wrapper
                .wrap_line(&[LineFragment::text("aa bbb cccc ddddd eeee")], px(72.))
                .collect::<Vec<_>>(),
            &[
                Boundary::new(7, 0),
                Boundary::new(12, 0),
                Boundary::new(18, 0)
            ],
        );
        assert_eq!(
            wrapper
                .wrap_line(&[LineFragment::text("aaa aaaaaaaaaaaaaaaaaa")], px(72.0))
                .collect::<Vec<_>>(),
            &[
                Boundary::new(4, 0),
                Boundary::new(11, 0),
                Boundary::new(18, 0)
            ],
        );
        assert_eq!(
            wrapper
                .wrap_line(&[LineFragment::text("     aaaaaaa")], px(72.))
                .collect::<Vec<_>>(),
            &[
                Boundary::new(7, 5),
                Boundary::new(9, 5),
                Boundary::new(11, 5),
            ]
        );
        assert_eq!(
            wrapper
                .wrap_line(
                    &[LineFragment::text("                            ")],
                    px(72.)
                )
                .collect::<Vec<_>>(),
            &[
                Boundary::new(7, 0),
                Boundary::new(14, 0),
                Boundary::new(21, 0)
            ]
        );
        assert_eq!(
            wrapper
                .wrap_line(&[LineFragment::text("          aaaaaaaaaaaaaa")], px(72.))
                .collect::<Vec<_>>(),
            &[
                Boundary::new(7, 0),
                Boundary::new(14, 3),
                Boundary::new(18, 3),
                Boundary::new(22, 3),
            ]
        );

        // Test wrapping multiple text fragments
        assert_eq!(
            wrapper
                .wrap_line(
                    &[
                        LineFragment::text("aa bbb "),
                        LineFragment::text("cccc ddddd eeee")
                    ],
                    px(72.)
                )
                .collect::<Vec<_>>(),
            &[
                Boundary::new(7, 0),
                Boundary::new(12, 0),
                Boundary::new(18, 0)
            ],
        );

        // Test wrapping with a mix of text and element fragments
        assert_eq!(
            wrapper
                .wrap_line(
                    &[
                        LineFragment::text("aa "),
                        LineFragment::element(px(20.), 1),
                        LineFragment::text(" bbb "),
                        LineFragment::element(px(30.), 1),
                        LineFragment::text(" cccc")
                    ],
                    px(72.)
                )
                .collect::<Vec<_>>(),
            &[
                Boundary::new(5, 0),
                Boundary::new(9, 0),
                Boundary::new(11, 0)
            ],
        );

        // Test with element at the beginning and text afterward
        assert_eq!(
            wrapper
                .wrap_line(
                    &[
                        LineFragment::element(px(50.), 1),
                        LineFragment::text(" aaaa bbbb cccc dddd")
                    ],
                    px(72.)
                )
                .collect::<Vec<_>>(),
            &[
                Boundary::new(2, 0),
                Boundary::new(7, 0),
                Boundary::new(12, 0),
                Boundary::new(17, 0)
            ],
        );

        // Test with a large element that forces wrapping by itself
        assert_eq!(
            wrapper
                .wrap_line(
                    &[
                        LineFragment::text("short text "),
                        LineFragment::element(px(100.), 1),
                        LineFragment::text(" more text")
                    ],
                    px(72.)
                )
                .collect::<Vec<_>>(),
            &[
                Boundary::new(6, 0),
                Boundary::new(11, 0),
                Boundary::new(12, 0),
                Boundary::new(18, 0)
            ],
        );
    }

    #[test]
    fn test_truncate_line() {
        let mut wrapper = build_wrapper();

        fn perform_test(
            wrapper: &mut LineWrapper,
            text: &'static str,
            result: &'static str,
            ellipsis: &str,
        ) {
            let dummy_run_lens = vec![text.len()];
            let mut dummy_runs = generate_test_runs(&dummy_run_lens);
            assert_eq!(
                wrapper.truncate_line(text.into(), px(220.), ellipsis, &mut dummy_runs),
                result
            );
            assert_eq!(dummy_runs.first().unwrap().len, result.len());
        }

        perform_test(
            &mut wrapper,
            "aa bbb cccc ddddd eeee ffff gggg",
            "aa bbb cccc ddddd eeee",
            "",
        );
        perform_test(
            &mut wrapper,
            "aa bbb cccc ddddd eeee ffff gggg",
            "aa bbb cccc ddddd eeeâ€¦",
            "â€¦",
        );
        perform_test(
            &mut wrapper,
            "aa bbb cccc ddddd eeee ffff gggg",
            "aa bbb cccc dddd......",
            "......",
        );
    }

    #[test]
    fn test_truncate_multiple_runs() {
        let mut wrapper = build_wrapper();

        fn perform_test(
            wrapper: &mut LineWrapper,
            text: &'static str,
            result: &str,
            run_lens: &[usize],
            result_run_len: &[usize],
            line_width: Pixels,
        ) {
            let mut dummy_runs = generate_test_runs(run_lens);
            assert_eq!(
                wrapper.truncate_line(text.into(), line_width, "â€¦", &mut dummy_runs),
                result
            );
            for (run, result_len) in dummy_runs.iter().zip(result_run_len) {
                assert_eq!(run.len, *result_len);
            }
        }
        // Case 0: Normal
        // Text: abcdefghijkl
        // Runs: Run0 { len: 12, ... }
        //
        // Truncate res: abcdâ€¦ (truncate_at = 4)
        // Run res: Run0 { string: abcdâ€¦, len: 7, ... }
        perform_test(&mut wrapper, "abcdefghijkl", "abcdâ€¦", &[12], &[7], px(50.));
        // Case 1: Drop some runs
        // Text: abcdefghijkl
        // Runs: Run0 { len: 4, ... }, Run1 { len: 4, ... }, Run2 { len: 4, ... }
        //
        // Truncate res: abcdefâ€¦ (truncate_at = 6)
        // Runs res: Run0 { string: abcd, len: 4, ... }, Run1 { string: efâ€¦, len:
        // 5, ... }
        perform_test(
            &mut wrapper,
            "abcdefghijkl",
            "abcdefâ€¦",
            &[4, 4, 4],
            &[4, 5],
            px(70.),
        );
        // Case 2: Truncate at start of some run
        // Text: abcdefghijkl
        // Runs: Run0 { len: 4, ... }, Run1 { len: 4, ... }, Run2 { len: 4, ... }
        //
        // Truncate res: abcdefghâ€¦ (truncate_at = 8)
        // Runs res: Run0 { string: abcd, len: 4, ... }, Run1 { string: efgh, len:
        // 4, ... }, Run2 { string: â€¦, len: 3, ... }
        perform_test(
            &mut wrapper,
            "abcdefghijkl",
            "abcdefghâ€¦",
            &[4, 4, 4],
            &[4, 4, 3],
            px(90.),
        );
    }

    #[test]
    fn test_update_run_after_truncation() {
        fn perform_test(result: &str, run_lens: &[usize], result_run_lens: &[usize]) {
            let mut dummy_runs = generate_test_runs(run_lens);
            update_runs_after_truncation(result, "â€¦", &mut dummy_runs);
            for (run, result_len) in dummy_runs.iter().zip(result_run_lens) {
                assert_eq!(run.len, *result_len);
            }
        }
        // Case 0: Normal
        // Text: abcdefghijkl
        // Runs: Run0 { len: 12, ... }
        //
        // Truncate res: abcdâ€¦ (truncate_at = 4)
        // Run res: Run0 { string: abcdâ€¦, len: 7, ... }
        perform_test("abcdâ€¦", &[12], &[7]);
        // Case 1: Drop some runs
        // Text: abcdefghijkl
        // Runs: Run0 { len: 4, ... }, Run1 { len: 4, ... }, Run2 { len: 4, ... }
        //
        // Truncate res: abcdefâ€¦ (truncate_at = 6)
        // Runs res: Run0 { string: abcd, len: 4, ... }, Run1 { string: efâ€¦, len:
        // 5, ... }
        perform_test("abcdefâ€¦", &[4, 4, 4], &[4, 5]);
        // Case 2: Truncate at start of some run
        // Text: abcdefghijkl
        // Runs: Run0 { len: 4, ... }, Run1 { len: 4, ... }, Run2 { len: 4, ... }
        //
        // Truncate res: abcdefghâ€¦ (truncate_at = 8)
        // Runs res: Run0 { string: abcd, len: 4, ... }, Run1 { string: efgh, len:
        // 4, ... }, Run2 { string: â€¦, len: 3, ... }
        perform_test("abcdefghâ€¦", &[4, 4, 4], &[4, 4, 3]);
    }

    #[test]
    fn test_is_word_char() {
        #[track_caller]
        fn assert_word(word: &str) {
            for c in word.chars() {
                assert!(LineWrapper::is_word_char(c), "assertion failed for '{}'", c);
            }
        }

        #[track_caller]
        fn assert_not_word(word: &str) {
            let found = word.chars().any(|c| !LineWrapper::is_word_char(c));
            assert!(found, "assertion failed for '{}'", word);
        }

        assert_word("Hello123");
        assert_word("non-English");
        assert_word("var_name");
        assert_word("123456");
        assert_word("3.1415");
        assert_word("10^2");
        assert_word("1~2");
        assert_word("100%");
        assert_word("@mention");
        assert_word("#hashtag");
        assert_word("$variable");
        assert_word("moreâ‹¯");

        // Space
        assert_not_word("foo bar");

        // URL case
        assert_word("https://github.com/CodeOrbit-industries/CodeOrbit/");
        assert_word("github.com");
        assert_word("a=1&b=2");

        // Latin-1 Supplement
        assert_word("Ã€ÃÃ‚ÃƒÃ„Ã…Ã†Ã‡ÃˆÃ‰ÃŠÃ‹ÃŒÃÃŽÃ");
        // Latin Extended-A
        assert_word("Ä€ÄÄ‚ÄƒÄ„Ä…Ä†Ä‡ÄˆÄ‰ÄŠÄ‹ÄŒÄÄŽÄ");
        // Latin Extended-B
        assert_word("Æ€ÆÆ‚ÆƒÆ„Æ…Æ†Æ‡ÆˆÆ‰ÆŠÆ‹ÆŒÆÆŽÆ");
        // Cyrillic
        assert_word("ÐÐ‘Ð’Ð“Ð”Ð•Ð–Ð—Ð˜Ð™ÐšÐ›ÐœÐÐžÐŸ");

        // non-word characters
        assert_not_word("ä½ å¥½");
        assert_not_word("ì•ˆë…•í•˜ì„¸ìš”");
        assert_not_word("ã“ã‚“ã«ã¡ã¯");
        assert_not_word("ðŸ˜€ðŸ˜ðŸ˜‚");
        assert_not_word("()[]{}<>");
    }

    // For compatibility with the test macro
    #[cfg(target_os = "macos")]
    use crate as gpui;

    // These seem to vary wildly based on the text system.
    #[cfg(target_os = "macos")]
    #[crate::test]
    fn test_wrap_shaped_line(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let text_system = WindowTextSystem::new(cx.text_system().clone());

            let normal = TextRun {
                len: 0,
                font: font("Helvetica"),
                color: Default::default(),
                underline: Default::default(),
                strikethrough: None,
                background_color: None,
            };
            let bold = TextRun {
                len: 0,
                font: font("Helvetica").bold(),
                color: Default::default(),
                underline: Default::default(),
                strikethrough: None,
                background_color: None,
            };

            let text = "aa bbb cccc ddddd eeee".into();
            let lines = text_system
                .shape_text(
                    text,
                    px(16.),
                    &[
                        normal.with_len(4),
                        bold.with_len(5),
                        normal.with_len(6),
                        bold.with_len(1),
                        normal.with_len(7),
                    ],
                    Some(px(72.)),
                    None,
                )
                .unwrap();

            assert_eq!(
                lines[0].layout.wrap_boundaries(),
                &[
                    WrapBoundary {
                        run_ix: 0,
                        glyph_ix: 7
                    },
                    WrapBoundary {
                        run_ix: 0,
                        glyph_ix: 12
                    },
                    WrapBoundary {
                        run_ix: 0,
                        glyph_ix: 18
                    }
                ],
            );
        });
    }
}
