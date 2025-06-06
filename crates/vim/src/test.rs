﻿mod neovim_backed_test_context;
mod neovim_connection;
mod vim_test_context;

use std::time::Duration;

use collections::HashMap;
use command_palette::CommandPalette;
use editor::{
    DisplayPoint, Editor, EditorMode, MultiBuffer, actions::DeleteLine, display_map::DisplayRow,
    test::editor_test_context::EditorTestContext,
};
use futures::StreamExt;
use gpui::{KeyBinding, Modifiers, MouseButton, TestAppContext};
use language::Point;
pub use neovim_backed_test_context::*;
use settings::SettingsStore;
pub use vim_test_context::*;

use indoc::indoc;
use search::BufferSearchBar;
use workspace::WorkspaceSettings;

use crate::{PushSneak, PushSneakBackward, insert::NormalBefore, motion, state::Mode};

#[gpui::test]
async fn test_initially_disabled(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, false).await;
    cx.simulate_keystrokes("h j k l");
    cx.assert_editor_state("hjklË‡");
}

#[gpui::test]
async fn test_neovim(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.simulate_shared_keystrokes("i").await;
    cx.shared_state().await.assert_matches();
    cx.simulate_shared_keystrokes("shift-t e s t space t e s t escape 0 d w")
        .await;
    cx.shared_state().await.assert_matches();
    cx.assert_editor_state("Ë‡test");
}

#[gpui::test]
async fn test_toggle_through_settings(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.simulate_keystrokes("i");
    assert_eq!(cx.mode(), Mode::Insert);

    // Editor acts as though vim is disabled
    cx.disable_vim();
    cx.simulate_keystrokes("h j k l");
    cx.assert_editor_state("hjklË‡");

    // Selections aren't changed if editor is blurred but vim-mode is still disabled.
    cx.cx.set_state("Â«hjklË‡Â»");
    cx.assert_editor_state("Â«hjklË‡Â»");
    cx.update_editor(|_, window, _cx| window.blur());
    cx.assert_editor_state("Â«hjklË‡Â»");
    cx.update_editor(|_, window, cx| cx.focus_self(window));
    cx.assert_editor_state("Â«hjklË‡Â»");

    // Enabling dynamically sets vim mode again and restores normal mode
    cx.enable_vim();
    assert_eq!(cx.mode(), Mode::Normal);
    cx.simulate_keystrokes("h h h l");
    assert_eq!(cx.buffer_text(), "hjkl".to_owned());
    cx.assert_editor_state("hË‡jkl");
    cx.simulate_keystrokes("i T e s t");
    cx.assert_editor_state("hTestË‡jkl");

    // Disabling and enabling resets to normal mode
    assert_eq!(cx.mode(), Mode::Insert);
    cx.disable_vim();
    cx.enable_vim();
    assert_eq!(cx.mode(), Mode::Normal);
}

#[gpui::test]
async fn test_cancel_selection(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(
        indoc! {"The quick brown fox juË‡mps over the lazy dog"},
        Mode::Normal,
    );
    // jumps
    cx.simulate_keystrokes("v l l");
    cx.assert_editor_state("The quick brown fox juÂ«mpsË‡Â» over the lazy dog");

    cx.simulate_keystrokes("escape");
    cx.assert_editor_state("The quick brown fox jumpË‡s over the lazy dog");

    // go back to the same selection state
    cx.simulate_keystrokes("v h h");
    cx.assert_editor_state("The quick brown fox juÂ«Ë‡mpsÂ» over the lazy dog");

    // Ctrl-[ should behave like Esc
    cx.simulate_keystrokes("ctrl-[");
    cx.assert_editor_state("The quick brown fox juË‡mps over the lazy dog");
}

#[gpui::test]
async fn test_buffer_search(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(
        indoc! {"
            The quick brown
            fox juË‡mps over
            the lazy dog"},
        Mode::Normal,
    );
    cx.simulate_keystrokes("/");

    let search_bar = cx.workspace(|workspace, _, cx| {
        workspace
            .active_pane()
            .read(cx)
            .toolbar()
            .read(cx)
            .item_of_type::<BufferSearchBar>()
            .expect("Buffer search bar should be deployed")
    });

    cx.update_entity(search_bar, |bar, _, cx| {
        assert_eq!(bar.query(cx), "");
    })
}

#[gpui::test]
async fn test_count_down(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(indoc! {"aË‡a\nbb\ncc\ndd\nee"}, Mode::Normal);
    cx.simulate_keystrokes("2 down");
    cx.assert_editor_state("aa\nbb\ncË‡c\ndd\nee");
    cx.simulate_keystrokes("9 down");
    cx.assert_editor_state("aa\nbb\ncc\ndd\neË‡e");
}

#[gpui::test]
async fn test_end_of_document_710(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // goes to end by default
    cx.set_state(indoc! {"aË‡a\nbb\ncc"}, Mode::Normal);
    cx.simulate_keystrokes("shift-g");
    cx.assert_editor_state("aa\nbb\ncË‡c");

    // can go to line 1 (https://github.com/CodeOrbit-industries/CodeOrbit/issues/5812)
    cx.simulate_keystrokes("1 shift-g");
    cx.assert_editor_state("aË‡a\nbb\ncc");
}

#[gpui::test]
async fn test_end_of_line_with_times(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // goes to current line end
    cx.set_state(indoc! {"Ë‡aa\nbb\ncc"}, Mode::Normal);
    cx.simulate_keystrokes("$");
    cx.assert_editor_state("aË‡a\nbb\ncc");

    // goes to next line end
    cx.simulate_keystrokes("2 $");
    cx.assert_editor_state("aa\nbË‡b\ncc");

    // try to exceed the final line.
    cx.simulate_keystrokes("4 $");
    cx.assert_editor_state("aa\nbb\ncË‡c");
}

#[gpui::test]
async fn test_indent_outdent(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // works in normal mode
    cx.set_state(indoc! {"aa\nbË‡b\ncc"}, Mode::Normal);
    cx.simulate_keystrokes("> >");
    cx.assert_editor_state("aa\n    bË‡b\ncc");
    cx.simulate_keystrokes("< <");
    cx.assert_editor_state("aa\nbË‡b\ncc");

    // works in visual mode
    cx.simulate_keystrokes("shift-v down >");
    cx.assert_editor_state("aa\n    bË‡b\n    cc");

    // works as operator
    cx.set_state("aa\nbË‡b\ncc\n", Mode::Normal);
    cx.simulate_keystrokes("> j");
    cx.assert_editor_state("aa\n    bË‡b\n    cc\n");
    cx.simulate_keystrokes("< k");
    cx.assert_editor_state("aa\nbË‡b\n    cc\n");
    cx.simulate_keystrokes("> i p");
    cx.assert_editor_state("    aa\n    bË‡b\n        cc\n");
    cx.simulate_keystrokes("< i p");
    cx.assert_editor_state("aa\nbË‡b\n    cc\n");
    cx.simulate_keystrokes("< i p");
    cx.assert_editor_state("aa\nbË‡b\ncc\n");

    cx.set_state("Ë‡aa\nbb\ncc\n", Mode::Normal);
    cx.simulate_keystrokes("> 2 j");
    cx.assert_editor_state("    Ë‡aa\n    bb\n    cc\n");

    cx.set_state("aa\nbb\nË‡cc\n", Mode::Normal);
    cx.simulate_keystrokes("> 2 k");
    cx.assert_editor_state("    aa\n    bb\n    Ë‡cc\n");

    // works with repeat
    cx.set_state("a\nb\nccË‡c\n", Mode::Normal);
    cx.simulate_keystrokes("> 2 k");
    cx.assert_editor_state("    a\n    b\n    ccË‡c\n");
    cx.simulate_keystrokes(".");
    cx.assert_editor_state("        a\n        b\n        ccË‡c\n");
    cx.simulate_keystrokes("v k <");
    cx.assert_editor_state("        a\n    bË‡\n    ccc\n");
    cx.simulate_keystrokes(".");
    cx.assert_editor_state("        a\nbË‡\nccc\n");
}

#[gpui::test]
async fn test_escape_command_palette(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("aË‡bc\n", Mode::Normal);
    cx.simulate_keystrokes("i cmd-shift-p");

    assert!(
        cx.workspace(|workspace, _, cx| workspace.active_modal::<CommandPalette>(cx).is_some())
    );
    cx.simulate_keystrokes("escape");
    cx.run_until_parked();
    assert!(
        !cx.workspace(|workspace, _, cx| workspace.active_modal::<CommandPalette>(cx).is_some())
    );
    cx.assert_state("aË‡bc\n", Mode::Insert);
}

#[gpui::test]
async fn test_escape_cancels(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("aË‡bË‡c", Mode::Normal);
    cx.simulate_keystrokes("escape");

    cx.assert_state("aË‡bc", Mode::Normal);
}

#[gpui::test]
async fn test_selection_on_search(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(indoc! {"aa\nbË‡b\ncc\ncc\ncc\n"}, Mode::Normal);
    cx.simulate_keystrokes("/ c c");

    let search_bar = cx.workspace(|workspace, _, cx| {
        workspace
            .active_pane()
            .read(cx)
            .toolbar()
            .read(cx)
            .item_of_type::<BufferSearchBar>()
            .expect("Buffer search bar should be deployed")
    });

    cx.update_entity(search_bar, |bar, _, cx| {
        assert_eq!(bar.query(cx), "cc");
    });

    cx.update_editor(|editor, window, cx| {
        let highlights = editor.all_text_background_highlights(window, cx);
        assert_eq!(3, highlights.len());
        assert_eq!(
            DisplayPoint::new(DisplayRow(2), 0)..DisplayPoint::new(DisplayRow(2), 2),
            highlights[0].0
        )
    });
    cx.simulate_keystrokes("enter");

    cx.assert_state(indoc! {"aa\nbb\nË‡cc\ncc\ncc\n"}, Mode::Normal);
    cx.simulate_keystrokes("n");
    cx.assert_state(indoc! {"aa\nbb\ncc\nË‡cc\ncc\n"}, Mode::Normal);
    cx.simulate_keystrokes("shift-n");
    cx.assert_state(indoc! {"aa\nbb\nË‡cc\ncc\ncc\n"}, Mode::Normal);
}

#[gpui::test]
async fn test_word_characters(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new_typescript(cx).await;
    cx.set_state(
        indoc! { "
        class A {
            #Ë‡goop = 99;
            $Ë‡goop () { return this.#gË‡oop };
        };
        console.log(new A().$gooË‡p())
    "},
        Mode::Normal,
    );
    cx.simulate_keystrokes("v i w");
    cx.assert_state(
        indoc! {"
        class A {
            Â«#goopË‡Â» = 99;
            Â«$goopË‡Â» () { return this.Â«#goopË‡Â» };
        };
        console.log(new A().Â«$goopË‡Â»())
    "},
        Mode::Visual,
    )
}

#[gpui::test]
async fn test_kebab_case(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new_html(cx).await;
    cx.set_state(
        indoc! { r#"
            <div><a class="bg-rË‡ed"></a></div>
            "#},
        Mode::Normal,
    );
    cx.simulate_keystrokes("v i w");
    cx.assert_state(
        indoc! { r#"
        <div><a class="bg-Â«redË‡Â»"></a></div>
        "#
        },
        Mode::Visual,
    )
}

#[gpui::test]
async fn test_join_lines(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state(indoc! {"
      Ë‡one
      two
      three
      four
      five
      six
      "})
        .await;
    cx.simulate_shared_keystrokes("shift-j").await;
    cx.shared_state().await.assert_eq(indoc! {"
          oneË‡ two
          three
          four
          five
          six
          "});
    cx.simulate_shared_keystrokes("3 shift-j").await;
    cx.shared_state().await.assert_eq(indoc! {"
          one two threeË‡ four
          five
          six
          "});

    cx.set_shared_state(indoc! {"
      Ë‡one
      two
      three
      four
      five
      six
      "})
        .await;
    cx.simulate_shared_keystrokes("j v 3 j shift-j").await;
    cx.shared_state().await.assert_eq(indoc! {"
      one
      two three fourË‡ five
      six
      "});

    cx.set_shared_state(indoc! {"
      Ë‡one
      two
      three
      four
      five
      six
      "})
        .await;
    cx.simulate_shared_keystrokes("g shift-j").await;
    cx.shared_state().await.assert_eq(indoc! {"
          oneË‡two
          three
          four
          five
          six
          "});
    cx.simulate_shared_keystrokes("3 g shift-j").await;
    cx.shared_state().await.assert_eq(indoc! {"
          onetwothreeË‡four
          five
          six
          "});

    cx.set_shared_state(indoc! {"
      Ë‡one
      two
      three
      four
      five
      six
      "})
        .await;
    cx.simulate_shared_keystrokes("j v 3 j g shift-j").await;
    cx.shared_state().await.assert_eq(indoc! {"
      one
      twothreefourË‡five
      six
      "});
}

#[cfg(target_os = "macos")]
#[gpui::test]
async fn test_wrapped_lines(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_wrap(12).await;
    // tests line wrap as follows:
    //  1: twelve char
    //     twelve char
    //  2: twelve char
    cx.set_shared_state(indoc! { "
        tË‡welve char twelve char
        twelve char
    "})
        .await;
    cx.simulate_shared_keystrokes("j").await;
    cx.shared_state().await.assert_eq(indoc! {"
        twelve char twelve char
        tË‡welve char
    "});
    cx.simulate_shared_keystrokes("k").await;
    cx.shared_state().await.assert_eq(indoc! {"
        tË‡welve char twelve char
        twelve char
    "});
    cx.simulate_shared_keystrokes("g j").await;
    cx.shared_state().await.assert_eq(indoc! {"
        twelve char tË‡welve char
        twelve char
    "});
    cx.simulate_shared_keystrokes("g j").await;
    cx.shared_state().await.assert_eq(indoc! {"
        twelve char twelve char
        tË‡welve char
    "});

    cx.simulate_shared_keystrokes("g k").await;
    cx.shared_state().await.assert_eq(indoc! {"
        twelve char tË‡welve char
        twelve char
    "});

    cx.simulate_shared_keystrokes("g ^").await;
    cx.shared_state().await.assert_eq(indoc! {"
        twelve char Ë‡twelve char
        twelve char
    "});

    cx.simulate_shared_keystrokes("^").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Ë‡twelve char twelve char
        twelve char
    "});

    cx.simulate_shared_keystrokes("g $").await;
    cx.shared_state().await.assert_eq(indoc! {"
        twelve charË‡ twelve char
        twelve char
    "});
    cx.simulate_shared_keystrokes("$").await;
    cx.shared_state().await.assert_eq(indoc! {"
        twelve char twelve chaË‡r
        twelve char
    "});

    cx.set_shared_state(indoc! { "
        tË‡welve char twelve char
        twelve char
    "})
        .await;
    cx.simulate_shared_keystrokes("enter").await;
    cx.shared_state().await.assert_eq(indoc! {"
            twelve char twelve char
            Ë‡twelve char
        "});

    cx.set_shared_state(indoc! { "
        twelve char
        tË‡welve char twelve char
        twelve char
    "})
        .await;
    cx.simulate_shared_keystrokes("o o escape").await;
    cx.shared_state().await.assert_eq(indoc! {"
        twelve char
        twelve char twelve char
        Ë‡o
        twelve char
    "});

    cx.set_shared_state(indoc! { "
        twelve char
        tË‡welve char twelve char
        twelve char
    "})
        .await;
    cx.simulate_shared_keystrokes("shift-a a escape").await;
    cx.shared_state().await.assert_eq(indoc! {"
        twelve char
        twelve char twelve charË‡a
        twelve char
    "});
    cx.simulate_shared_keystrokes("shift-i i escape").await;
    cx.shared_state().await.assert_eq(indoc! {"
        twelve char
        Ë‡itwelve char twelve chara
        twelve char
    "});
    cx.simulate_shared_keystrokes("shift-d").await;
    cx.shared_state().await.assert_eq(indoc! {"
        twelve char
        Ë‡
        twelve char
    "});

    cx.set_shared_state(indoc! { "
        twelve char
        twelve char tË‡welve char
        twelve char
    "})
        .await;
    cx.simulate_shared_keystrokes("shift-o o escape").await;
    cx.shared_state().await.assert_eq(indoc! {"
        twelve char
        Ë‡o
        twelve char twelve char
        twelve char
    "});

    // line wraps as:
    // fourteen ch
    // ar
    // fourteen ch
    // ar
    cx.set_shared_state(indoc! { "
        fourteen chaË‡r
        fourteen char
    "})
        .await;

    cx.simulate_shared_keystrokes("d i w").await;
    cx.shared_state().await.assert_eq(indoc! {"
        fourteenË‡â€¢
        fourteen char
    "});
    cx.simulate_shared_keystrokes("j shift-f e f r").await;
    cx.shared_state().await.assert_eq(indoc! {"
        fourteenâ€¢
        fourteen chaË‡r
    "});
}

#[gpui::test]
async fn test_folds(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;
    cx.set_neovim_option("foldmethod=manual").await;

    cx.set_shared_state(indoc! { "
        fn boop() {
          Ë‡barp()
          bazp()
        }
    "})
        .await;
    cx.simulate_shared_keystrokes("shift-v j z f").await;

    // visual display is now:
    // fn boop () {
    //  [FOLDED]
    // }

    // TODO: this should not be needed but currently zf does not
    // return to normal mode.
    cx.simulate_shared_keystrokes("escape").await;

    // skip over fold downward
    cx.simulate_shared_keystrokes("g g").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Ë‡fn boop() {
          barp()
          bazp()
        }
    "});

    cx.simulate_shared_keystrokes("j j").await;
    cx.shared_state().await.assert_eq(indoc! {"
        fn boop() {
          barp()
          bazp()
        Ë‡}
    "});

    // skip over fold upward
    cx.simulate_shared_keystrokes("2 k").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Ë‡fn boop() {
          barp()
          bazp()
        }
    "});

    // yank the fold
    cx.simulate_shared_keystrokes("down y y").await;
    cx.shared_clipboard()
        .await
        .assert_eq("  barp()\n  bazp()\n");

    // re-open
    cx.simulate_shared_keystrokes("z o").await;
    cx.shared_state().await.assert_eq(indoc! {"
        fn boop() {
        Ë‡  barp()
          bazp()
        }
    "});
}

#[gpui::test]
async fn test_folds_panic(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;
    cx.set_neovim_option("foldmethod=manual").await;

    cx.set_shared_state(indoc! { "
        fn boop() {
          Ë‡barp()
          bazp()
        }
    "})
        .await;
    cx.simulate_shared_keystrokes("shift-v j z f").await;
    cx.simulate_shared_keystrokes("escape").await;
    cx.simulate_shared_keystrokes("g g").await;
    cx.simulate_shared_keystrokes("5 d j").await;
    cx.shared_state().await.assert_eq("Ë‡");
    cx.set_shared_state(indoc! {"
        fn boop() {
          Ë‡barp()
          bazp()
        }
    "})
        .await;
    cx.simulate_shared_keystrokes("shift-v j j z f").await;
    cx.simulate_shared_keystrokes("escape").await;
    cx.simulate_shared_keystrokes("shift-g shift-v").await;
    cx.shared_state().await.assert_eq(indoc! {"
        fn boop() {
          barp()
          bazp()
        }
        Ë‡"});
}

#[gpui::test]
async fn test_clear_counts(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state(indoc! {"
        The quick brown
        fox juË‡mps over
        the lazy dog"})
        .await;

    cx.simulate_shared_keystrokes("4 escape 3 d l").await;
    cx.shared_state().await.assert_eq(indoc! {"
        The quick brown
        fox juË‡ over
        the lazy dog"});
}

#[gpui::test]
async fn test_zero(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state(indoc! {"
        The quË‡ick brown
        fox jumps over
        the lazy dog"})
        .await;

    cx.simulate_shared_keystrokes("0").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Ë‡The quick brown
        fox jumps over
        the lazy dog"});

    cx.simulate_shared_keystrokes("1 0 l").await;
    cx.shared_state().await.assert_eq(indoc! {"
        The quick Ë‡brown
        fox jumps over
        the lazy dog"});
}

#[gpui::test]
async fn test_selection_goal(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state(indoc! {"
        ;;Ë‡;
        Lorem Ipsum"})
        .await;

    cx.simulate_shared_keystrokes("a down up ; down up").await;
    cx.shared_state().await.assert_eq(indoc! {"
        ;;;;Ë‡
        Lorem Ipsum"});
}

#[cfg(target_os = "macos")]
#[gpui::test]
async fn test_wrapped_motions(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_wrap(12).await;

    cx.set_shared_state(indoc! {"
                aaË‡aa
                ðŸ˜ƒðŸ˜ƒ"
    })
    .await;
    cx.simulate_shared_keystrokes("j").await;
    cx.shared_state().await.assert_eq(indoc! {"
                aaaa
                ðŸ˜ƒË‡ðŸ˜ƒ"
    });

    cx.set_shared_state(indoc! {"
                123456789012aaË‡aa
                123456789012ðŸ˜ƒðŸ˜ƒ"
    })
    .await;
    cx.simulate_shared_keystrokes("j").await;
    cx.shared_state().await.assert_eq(indoc! {"
        123456789012aaaa
        123456789012ðŸ˜ƒË‡ðŸ˜ƒ"
    });

    cx.set_shared_state(indoc! {"
                123456789012aaË‡aa
                123456789012ðŸ˜ƒðŸ˜ƒ"
    })
    .await;
    cx.simulate_shared_keystrokes("j").await;
    cx.shared_state().await.assert_eq(indoc! {"
        123456789012aaaa
        123456789012ðŸ˜ƒË‡ðŸ˜ƒ"
    });

    cx.set_shared_state(indoc! {"
        123456789012aaaaË‡aaaaaaaa123456789012
        wow
        123456789012ðŸ˜ƒðŸ˜ƒðŸ˜ƒðŸ˜ƒðŸ˜ƒðŸ˜ƒ123456789012"
    })
    .await;
    cx.simulate_shared_keystrokes("j j").await;
    cx.shared_state().await.assert_eq(indoc! {"
        123456789012aaaaaaaaaaaa123456789012
        wow
        123456789012ðŸ˜ƒðŸ˜ƒË‡ðŸ˜ƒðŸ˜ƒðŸ˜ƒðŸ˜ƒ123456789012"
    });
}

#[gpui::test]
async fn test_wrapped_delete_end_document(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_wrap(12).await;

    cx.set_shared_state(indoc! {"
                aaË‡aaaaaaaaaaaaaaaaaa
                bbbbbbbbbbbbbbbbbbbb
                cccccccccccccccccccc"
    })
    .await;
    cx.simulate_shared_keystrokes("d shift-g i z z z").await;
    cx.shared_state().await.assert_eq(indoc! {"
                zzzË‡"
    });
}

#[gpui::test]
async fn test_paragraphs_dont_wrap(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state(indoc! {"
        one
        Ë‡
        two"})
        .await;

    cx.simulate_shared_keystrokes("} }").await;
    cx.shared_state().await.assert_eq(indoc! {"
        one

        twË‡o"});

    cx.simulate_shared_keystrokes("{ { {").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Ë‡one

        two"});
}

#[gpui::test]
async fn test_select_all_issue_2170(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state(
        indoc! {"
        defmodule Test do
            def test(a, Ë‡[_, _] = b), do: IO.puts('hi')
        end
    "},
        Mode::Normal,
    );
    cx.simulate_keystrokes("g a");
    cx.assert_state(
        indoc! {"
        defmodule Test do
            def test(a, Â«[Ë‡Â»_, _] = b), do: IO.puts('hi')
        end
    "},
        Mode::Visual,
    );
}

#[gpui::test]
async fn test_jk(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "j k",
            NormalBefore,
            Some("vim_mode == insert"),
        )])
    });
    cx.neovim.exec("imap jk <esc>").await;

    cx.set_shared_state("Ë‡hello").await;
    cx.simulate_shared_keystrokes("i j o j k").await;
    cx.shared_state().await.assert_eq("jË‡ohello");
}

#[gpui::test]
async fn test_jk_delay(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "j k",
            NormalBefore,
            Some("vim_mode == insert"),
        )])
    });

    cx.set_state("Ë‡hello", Mode::Normal);
    cx.simulate_keystrokes("i j");
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.run_until_parked();
    cx.assert_state("Ë‡hello", Mode::Insert);
    cx.executor().advance_clock(Duration::from_millis(500));
    cx.run_until_parked();
    cx.assert_state("jË‡hello", Mode::Insert);
    cx.simulate_keystrokes("k j k");
    cx.assert_state("jË‡khello", Mode::Normal);
}

#[gpui::test]
async fn test_comma_w(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            ", w",
            motion::Down {
                display_lines: false,
            },
            Some("vim_mode == normal"),
        )])
    });
    cx.neovim.exec("map ,w j").await;

    cx.set_shared_state("Ë‡hello hello\nhello hello").await;
    cx.simulate_shared_keystrokes("f o ; , w").await;
    cx.shared_state()
        .await
        .assert_eq("hello hello\nhello hellË‡o");

    cx.set_shared_state("Ë‡hello hello\nhello hello").await;
    cx.simulate_shared_keystrokes("f o ; , i").await;
    cx.shared_state()
        .await
        .assert_eq("hellË‡o hello\nhello hello");
}

#[gpui::test]
async fn test_rename(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new_typescript(cx).await;

    cx.set_state("const beË‡fore = 2; console.log(before)", Mode::Normal);
    let def_range = cx.lsp_range("const Â«beforeË‡Â» = 2; console.log(before)");
    let tgt_range = cx.lsp_range("const before = 2; console.log(Â«beforeË‡Â»)");
    let mut prepare_request = cx.set_request_handler::<lsp::request::PrepareRenameRequest, _, _>(
        move |_, _, _| async move { Ok(Some(lsp::PrepareRenameResponse::Range(def_range))) },
    );
    let mut rename_request =
        cx.set_request_handler::<lsp::request::Rename, _, _>(move |url, params, _| async move {
            Ok(Some(lsp::WorkspaceEdit {
                changes: Some(
                    [(
                        url.clone(),
                        vec![
                            lsp::TextEdit::new(def_range, params.new_name.clone()),
                            lsp::TextEdit::new(tgt_range, params.new_name),
                        ],
                    )]
                    .into(),
                ),
                ..Default::default()
            }))
        });

    cx.simulate_keystrokes("c d");
    prepare_request.next().await.unwrap();
    cx.simulate_input("after");
    cx.simulate_keystrokes("enter");
    rename_request.next().await.unwrap();
    cx.assert_state("const afterË‡ = 2; console.log(after)", Mode::Normal)
}

// TODO: this test is flaky on our linux CI machines
#[cfg(target_os = "macos")]
#[gpui::test]
async fn test_remap(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    // test moving the cursor
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "g z",
            workspace::SendKeystrokes("l l l l".to_string()),
            None,
        )])
    });
    cx.set_state("Ë‡123456789", Mode::Normal);
    cx.simulate_keystrokes("g z");
    cx.assert_state("1234Ë‡56789", Mode::Normal);

    // test switching modes
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "g y",
            workspace::SendKeystrokes("i f o o escape l".to_string()),
            None,
        )])
    });
    cx.set_state("Ë‡123456789", Mode::Normal);
    cx.simulate_keystrokes("g y");
    cx.assert_state("fooË‡123456789", Mode::Normal);

    // test recursion
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "g x",
            workspace::SendKeystrokes("g z g y".to_string()),
            None,
        )])
    });
    cx.set_state("Ë‡123456789", Mode::Normal);
    cx.simulate_keystrokes("g x");
    cx.assert_state("1234fooË‡56789", Mode::Normal);

    cx.executor().allow_parking();

    // test command
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "g w",
            workspace::SendKeystrokes(": j enter".to_string()),
            None,
        )])
    });
    cx.set_state("Ë‡1234\n56789", Mode::Normal);
    cx.simulate_keystrokes("g w");
    cx.assert_state("1234Ë‡ 56789", Mode::Normal);

    // test leaving command
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "g u",
            workspace::SendKeystrokes("g w g z".to_string()),
            None,
        )])
    });
    cx.set_state("Ë‡1234\n56789", Mode::Normal);
    cx.simulate_keystrokes("g u");
    cx.assert_state("1234 567Ë‡89", Mode::Normal);

    // test leaving command
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "g t",
            workspace::SendKeystrokes("i space escape".to_string()),
            None,
        )])
    });
    cx.set_state("12Ë‡34", Mode::Normal);
    cx.simulate_keystrokes("g t");
    cx.assert_state("12Ë‡ 34", Mode::Normal);
}

#[gpui::test]
async fn test_undo(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state("hello quË‡oel world").await;
    cx.simulate_shared_keystrokes("v i w s c o escape u").await;
    cx.shared_state().await.assert_eq("hello Ë‡quoel world");
    cx.simulate_shared_keystrokes("ctrl-r").await;
    cx.shared_state().await.assert_eq("hello Ë‡co world");
    cx.simulate_shared_keystrokes("a o right l escape").await;
    cx.shared_state().await.assert_eq("hello cooË‡l world");
    cx.simulate_shared_keystrokes("u").await;
    cx.shared_state().await.assert_eq("hello cooË‡ world");
    cx.simulate_shared_keystrokes("u").await;
    cx.shared_state().await.assert_eq("hello cË‡o world");
    cx.simulate_shared_keystrokes("u").await;
    cx.shared_state().await.assert_eq("hello Ë‡quoel world");

    cx.set_shared_state("hello quË‡oel world").await;
    cx.simulate_shared_keystrokes("v i w ~ u").await;
    cx.shared_state().await.assert_eq("hello Ë‡quoel world");

    cx.set_shared_state("\nhello quË‡oel world\n").await;
    cx.simulate_shared_keystrokes("shift-v s c escape u").await;
    cx.shared_state().await.assert_eq("\nË‡hello quoel world\n");

    cx.set_shared_state(indoc! {"
        Ë‡1
        2
        3"})
        .await;

    cx.simulate_shared_keystrokes("ctrl-v shift-g ctrl-a").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Ë‡2
        3
        4"});

    cx.simulate_shared_keystrokes("u").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Ë‡1
        2
        3"});
}

#[gpui::test]
async fn test_mouse_selection(cx: &mut TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.set_state("Ë‡one two three", Mode::Normal);

    let start_point = cx.pixel_position("one twË‡o three");
    let end_point = cx.pixel_position("one Ë‡two three");

    cx.simulate_mouse_down(start_point, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(end_point, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(end_point, MouseButton::Left, Modifiers::none());

    cx.assert_state("one Â«Ë‡twoÂ» three", Mode::Visual)
}

#[gpui::test]
async fn test_lowercase_marks(cx: &mut TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state("line one\nline Ë‡two\nline three").await;
    cx.simulate_shared_keystrokes("m a l ' a").await;
    cx.shared_state()
        .await
        .assert_eq("line one\nË‡line two\nline three");
    cx.simulate_shared_keystrokes("` a").await;
    cx.shared_state()
        .await
        .assert_eq("line one\nline Ë‡two\nline three");

    cx.simulate_shared_keystrokes("^ d ` a").await;
    cx.shared_state()
        .await
        .assert_eq("line one\nË‡two\nline three");
}

#[gpui::test]
async fn test_lt_gt_marks(cx: &mut TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state(indoc!(
        "
        Line one
        Line two
        Line Ë‡three
        Line four
        Line five
    "
    ))
    .await;

    cx.simulate_shared_keystrokes("v j escape k k").await;

    cx.simulate_shared_keystrokes("' <").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Line one
        Line two
        Ë‡Line three
        Line four
        Line five
    "});

    cx.simulate_shared_keystrokes("` <").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Line one
        Line two
        Line Ë‡three
        Line four
        Line five
    "});

    cx.simulate_shared_keystrokes("' >").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Line one
        Line two
        Line three
        Ë‡Line four
        Line five
    "
    });

    cx.simulate_shared_keystrokes("` >").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Line one
        Line two
        Line three
        Line Ë‡four
        Line five
    "
    });

    cx.simulate_shared_keystrokes("v i w o escape").await;
    cx.simulate_shared_keystrokes("` >").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Line one
        Line two
        Line three
        Line fouË‡r
        Line five
    "
    });
    cx.simulate_shared_keystrokes("` <").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Line one
        Line two
        Line three
        Line Ë‡four
        Line five
    "
    });
}

#[gpui::test]
async fn test_caret_mark(cx: &mut TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state(indoc!(
        "
        Line one
        Line two
        Line three
        Ë‡Line four
        Line five
    "
    ))
    .await;

    cx.simulate_shared_keystrokes("c w shift-s t r a i g h t space t h i n g escape j j")
        .await;

    cx.simulate_shared_keystrokes("' ^").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Line one
        Line two
        Line three
        Ë‡Straight thing four
        Line five
    "
    });

    cx.simulate_shared_keystrokes("` ^").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Line one
        Line two
        Line three
        Straight thingË‡ four
        Line five
    "
    });

    cx.simulate_shared_keystrokes("k a ! escape k g i ?").await;
    cx.shared_state().await.assert_eq(indoc! {"
        Line one
        Line two
        Line three!?Ë‡
        Straight thing four
        Line five
    "
    });
}

#[cfg(target_os = "macos")]
#[gpui::test]
async fn test_dw_eol(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_wrap(12).await;
    cx.set_shared_state("twelve Ë‡char twelve char\ntwelve char")
        .await;
    cx.simulate_shared_keystrokes("d w").await;
    cx.shared_state()
        .await
        .assert_eq("twelve Ë‡twelve char\ntwelve char");
}

#[gpui::test]
async fn test_toggle_comments(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    let language = std::sync::Arc::new(language::Language::new(
        language::LanguageConfig {
            line_comments: vec!["// ".into(), "//! ".into(), "/// ".into()],
            ..Default::default()
        },
        Some(language::tree_sitter_rust::LANGUAGE.into()),
    ));
    cx.update_buffer(|buffer, cx| buffer.set_language(Some(language), cx));

    // works in normal model
    cx.set_state(
        indoc! {"
      Ë‡one
      two
      three
      "},
        Mode::Normal,
    );
    cx.simulate_keystrokes("g c c");
    cx.assert_state(
        indoc! {"
          // Ë‡one
          two
          three
          "},
        Mode::Normal,
    );

    // works in visual mode
    cx.simulate_keystrokes("v j g c");
    cx.assert_state(
        indoc! {"
          // // Ë‡one
          // two
          three
          "},
        Mode::Normal,
    );

    // works in visual line mode
    cx.simulate_keystrokes("shift-v j g c");
    cx.assert_state(
        indoc! {"
          // Ë‡one
          two
          three
          "},
        Mode::Normal,
    );

    // works with count
    cx.simulate_keystrokes("g c 2 j");
    cx.assert_state(
        indoc! {"
            // // Ë‡one
            // two
            // three
            "},
        Mode::Normal,
    );

    // works with motion object
    cx.simulate_keystrokes("shift-g");
    cx.simulate_keystrokes("g c g g");
    cx.assert_state(
        indoc! {"
            // one
            two
            three
            Ë‡"},
        Mode::Normal,
    );
}

#[gpui::test]
async fn test_find_multibyte(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state(r#"<label for="guests">Ë‡PoÄet hostÅ¯</label>"#)
        .await;

    cx.simulate_shared_keystrokes("c t < o escape").await;
    cx.shared_state()
        .await
        .assert_eq(r#"<label for="guests">Ë‡o</label>"#);
}

#[gpui::test]
async fn test_sneak(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;

    cx.update(|_window, cx| {
        cx.bind_keys([
            KeyBinding::new(
                "s",
                PushSneak { first_char: None },
                Some("vim_mode == normal"),
            ),
            KeyBinding::new(
                "shift-s",
                PushSneakBackward { first_char: None },
                Some("vim_mode == normal"),
            ),
            KeyBinding::new(
                "shift-s",
                PushSneakBackward { first_char: None },
                Some("vim_mode == visual"),
            ),
        ])
    });

    // Sneak forwards multibyte & multiline
    cx.set_state(
        indoc! {
            r#"<labelË‡ for="guests">
                    PoÄet hostÅ¯
                </label>"#
        },
        Mode::Normal,
    );
    cx.simulate_keystrokes("s t Å¯");
    cx.assert_state(
        indoc! {
            r#"<label for="guests">
                PoÄet hosË‡tÅ¯
            </label>"#
        },
        Mode::Normal,
    );

    // Visual sneak backwards multibyte & multiline
    cx.simulate_keystrokes("v S < l");
    cx.assert_state(
        indoc! {
            r#"Â«Ë‡<label for="guests">
                PoÄet hostÂ»Å¯
            </label>"#
        },
        Mode::Visual,
    );

    // Sneak backwards repeated
    cx.set_state(r#"11 12 13 Ë‡14"#, Mode::Normal);
    cx.simulate_keystrokes("S space 1");
    cx.assert_state(r#"11 12Ë‡ 13 14"#, Mode::Normal);
    cx.simulate_keystrokes(";");
    cx.assert_state(r#"11Ë‡ 12 13 14"#, Mode::Normal);
}

#[gpui::test]
async fn test_plus_minus(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state(indoc! {
        "one
           two
        thrË‡ee
    "})
        .await;

    cx.simulate_shared_keystrokes("-").await;
    cx.shared_state().await.assert_matches();
    cx.simulate_shared_keystrokes("-").await;
    cx.shared_state().await.assert_matches();
    cx.simulate_shared_keystrokes("+").await;
    cx.shared_state().await.assert_matches();
}

#[gpui::test]
async fn test_command_alias(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;
    cx.update_global(|store: &mut SettingsStore, cx| {
        store.update_user_settings::<WorkspaceSettings>(cx, |s| {
            let mut aliases = HashMap::default();
            aliases.insert("Q".to_string(), "upper".to_string());
            s.command_aliases = Some(aliases)
        });
    });

    cx.set_state("Ë‡hello world", Mode::Normal);
    cx.simulate_keystrokes(": Q");
    cx.set_state("Ë‡Hello world", Mode::Normal);
}

#[gpui::test]
async fn test_remap_adjacent_dog_cat(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;
    cx.update(|_, cx| {
        cx.bind_keys([
            KeyBinding::new(
                "d o g",
                workspace::SendKeystrokes("ðŸ¶".to_string()),
                Some("vim_mode == insert"),
            ),
            KeyBinding::new(
                "c a t",
                workspace::SendKeystrokes("ðŸ±".to_string()),
                Some("vim_mode == insert"),
            ),
        ])
    });
    cx.neovim.exec("imap dog ðŸ¶").await;
    cx.neovim.exec("imap cat ðŸ±").await;

    cx.set_shared_state("Ë‡").await;
    cx.simulate_shared_keystrokes("i d o g").await;
    cx.shared_state().await.assert_eq("ðŸ¶Ë‡");

    cx.set_shared_state("Ë‡").await;
    cx.simulate_shared_keystrokes("i d o d o g").await;
    cx.shared_state().await.assert_eq("doðŸ¶Ë‡");

    cx.set_shared_state("Ë‡").await;
    cx.simulate_shared_keystrokes("i d o c a t").await;
    cx.shared_state().await.assert_eq("doðŸ±Ë‡");
}

#[gpui::test]
async fn test_remap_nested_pineapple(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;
    cx.update(|_, cx| {
        cx.bind_keys([
            KeyBinding::new(
                "p i n",
                workspace::SendKeystrokes("ðŸ“Œ".to_string()),
                Some("vim_mode == insert"),
            ),
            KeyBinding::new(
                "p i n e",
                workspace::SendKeystrokes("ðŸŒ²".to_string()),
                Some("vim_mode == insert"),
            ),
            KeyBinding::new(
                "p i n e a p p l e",
                workspace::SendKeystrokes("ðŸ".to_string()),
                Some("vim_mode == insert"),
            ),
        ])
    });
    cx.neovim.exec("imap pin ðŸ“Œ").await;
    cx.neovim.exec("imap pine ðŸŒ²").await;
    cx.neovim.exec("imap pineapple ðŸ").await;

    cx.set_shared_state("Ë‡").await;
    cx.simulate_shared_keystrokes("i p i n").await;
    cx.executor().advance_clock(Duration::from_millis(1000));
    cx.run_until_parked();
    cx.shared_state().await.assert_eq("ðŸ“ŒË‡");

    cx.set_shared_state("Ë‡").await;
    cx.simulate_shared_keystrokes("i p i n e").await;
    cx.executor().advance_clock(Duration::from_millis(1000));
    cx.run_until_parked();
    cx.shared_state().await.assert_eq("ðŸŒ²Ë‡");

    cx.set_shared_state("Ë‡").await;
    cx.simulate_shared_keystrokes("i p i n e a p p l e").await;
    cx.shared_state().await.assert_eq("ðŸË‡");
}

#[gpui::test]
async fn test_remap_recursion(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new(
            "x",
            workspace::SendKeystrokes("\" _ x".to_string()),
            Some("VimControl"),
        )]);
        cx.bind_keys([KeyBinding::new(
            "y",
            workspace::SendKeystrokes("2 x".to_string()),
            Some("VimControl"),
        )])
    });
    cx.neovim.exec("noremap x \"_x").await;
    cx.neovim.exec("map y 2x").await;

    cx.set_shared_state("Ë‡hello").await;
    cx.simulate_shared_keystrokes("d l").await;
    cx.shared_clipboard().await.assert_eq("h");
    cx.simulate_shared_keystrokes("y").await;
    cx.shared_clipboard().await.assert_eq("h");
    cx.shared_state().await.assert_eq("Ë‡lo");
}

#[gpui::test]
async fn test_escape_while_waiting(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;
    cx.set_shared_state("Ë‡hi").await;
    cx.simulate_shared_keystrokes("\" + escape x").await;
    cx.shared_state().await.assert_eq("Ë‡i");
}

#[gpui::test]
async fn test_ctrl_w_override(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;
    cx.update(|_, cx| {
        cx.bind_keys([KeyBinding::new("ctrl-w", DeleteLine, None)]);
    });
    cx.neovim.exec("map <c-w> D").await;
    cx.set_shared_state("Ë‡hi").await;
    cx.simulate_shared_keystrokes("ctrl-w").await;
    cx.shared_state().await.assert_eq("Ë‡");
}

#[gpui::test]
async fn test_visual_indent_count(cx: &mut gpui::TestAppContext) {
    let mut cx = VimTestContext::new(cx, true).await;
    cx.set_state("Ë‡hi", Mode::Normal);
    cx.simulate_keystrokes("shift-v 3 >");
    cx.assert_state("            Ë‡hi", Mode::Normal);
    cx.simulate_keystrokes("shift-v 2 <");
    cx.assert_state("    Ë‡hi", Mode::Normal);
}

#[gpui::test]
async fn test_record_replay_recursion(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state("Ë‡hello world").await;
    cx.simulate_shared_keystrokes(">").await;
    cx.simulate_shared_keystrokes(".").await;
    cx.simulate_shared_keystrokes(".").await;
    cx.simulate_shared_keystrokes(".").await;
    cx.shared_state().await.assert_eq("Ë‡hello world");
}

#[gpui::test]
async fn test_blackhole_register(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state("Ë‡hello world").await;
    cx.simulate_shared_keystrokes("d i w \" _ d a w").await;
    cx.simulate_shared_keystrokes("p").await;
    cx.shared_state().await.assert_eq("hellË‡o");
}

#[gpui::test]
async fn test_sentence_backwards(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state("one\n\ntwo\nthree\nË‡\nfour").await;
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state()
        .await
        .assert_eq("one\n\nË‡two\nthree\n\nfour");

    cx.set_shared_state("hello.\n\n\nworË‡ld.").await;
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state().await.assert_eq("hello.\n\n\nË‡world.");
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state().await.assert_eq("hello.\n\nË‡\nworld.");
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state().await.assert_eq("Ë‡hello.\n\n\nworld.");

    cx.set_shared_state("hello. worlË‡d.").await;
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state().await.assert_eq("hello. Ë‡world.");
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state().await.assert_eq("Ë‡hello. world.");

    cx.set_shared_state(". helË‡lo.").await;
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state().await.assert_eq(". Ë‡hello.");
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state().await.assert_eq(". Ë‡hello.");

    cx.set_shared_state(indoc! {
        "{
            hello_world();
        Ë‡}"
    })
    .await;
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state().await.assert_eq(indoc! {
        "Ë‡{
            hello_world();
        }"
    });

    cx.set_shared_state(indoc! {
        "Hello! World..?

        \tHello! World... Ë‡"
    })
    .await;
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state().await.assert_eq(indoc! {
        "Hello! World..?

        \tHello! Ë‡World... "
    });
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state().await.assert_eq(indoc! {
        "Hello! World..?

        \tË‡Hello! World... "
    });
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state().await.assert_eq(indoc! {
        "Hello! World..?
        Ë‡
        \tHello! World... "
    });
    cx.simulate_shared_keystrokes("(").await;
    cx.shared_state().await.assert_eq(indoc! {
        "Hello! Ë‡World..?

        \tHello! World... "
    });
}

#[gpui::test]
async fn test_sentence_forwards(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state("helË‡lo.\n\n\nworld.").await;
    cx.simulate_shared_keystrokes(")").await;
    cx.shared_state().await.assert_eq("hello.\nË‡\n\nworld.");
    cx.simulate_shared_keystrokes(")").await;
    cx.shared_state().await.assert_eq("hello.\n\n\nË‡world.");
    cx.simulate_shared_keystrokes(")").await;
    cx.shared_state().await.assert_eq("hello.\n\n\nworldË‡.");

    cx.set_shared_state("helË‡lo.\n\n\nworld.").await;
}

#[gpui::test]
async fn test_ctrl_o_visual(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state("helloË‡ world.").await;
    cx.simulate_shared_keystrokes("i ctrl-o v b r l").await;
    cx.shared_state().await.assert_eq("Ë‡llllllworld.");
    cx.simulate_shared_keystrokes("ctrl-o v f w d").await;
    cx.shared_state().await.assert_eq("Ë‡orld.");
}

#[gpui::test]
async fn test_ctrl_o_position(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state("helË‡lo world.").await;
    cx.simulate_shared_keystrokes("i ctrl-o d i w").await;
    cx.shared_state().await.assert_eq("Ë‡ world.");
    cx.simulate_shared_keystrokes("ctrl-o p").await;
    cx.shared_state().await.assert_eq(" helloË‡world.");
}

#[gpui::test]
async fn test_ctrl_o_dot(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;

    cx.set_shared_state("heË‡llo world.").await;
    cx.simulate_shared_keystrokes("x i ctrl-o .").await;
    cx.shared_state().await.assert_eq("heË‡o world.");
    cx.simulate_shared_keystrokes("l l escape .").await;
    cx.shared_state().await.assert_eq("hellË‡llo world.");
}

#[gpui::test]
async fn test_folded_multibuffer_excerpts(cx: &mut gpui::TestAppContext) {
    VimTestContext::init(cx);
    cx.update(|cx| {
        VimTestContext::init_keybindings(true, cx);
    });
    let (editor, cx) = cx.add_window_view(|window, cx| {
        let multi_buffer = MultiBuffer::build_multi(
            [
                ("111\n222\n333\n444\n", vec![Point::row_range(0..2)]),
                ("aaa\nbbb\nccc\nddd\n", vec![Point::row_range(0..2)]),
                ("AAA\nBBB\nCCC\nDDD\n", vec![Point::row_range(0..2)]),
                ("one\ntwo\nthr\nfou\n", vec![Point::row_range(0..2)]),
            ],
            cx,
        );
        let mut editor = Editor::new(EditorMode::full(), multi_buffer.clone(), None, window, cx);

        let buffer_ids = multi_buffer.read(cx).excerpt_buffer_ids();
        // fold all but the second buffer, so that we test navigating between two
        // adjacent folded buffers, as well as folded buffers at the start and
        // end the multibuffer
        editor.fold_buffer(buffer_ids[0], cx);
        editor.fold_buffer(buffer_ids[2], cx);
        editor.fold_buffer(buffer_ids[3], cx);

        editor
    });
    let mut cx = EditorTestContext::for_editor_in(editor.clone(), cx).await;

    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        Ë‡[FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("j");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        Ë‡aaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("j");
    cx.simulate_keystroke("j");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        aaa
        bbb
        Ë‡[EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("j");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        Ë‡[FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("j");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        Ë‡[FOLDED]
        "
    });
    cx.simulate_keystroke("k");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        Ë‡[FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("k");
    cx.simulate_keystroke("k");
    cx.simulate_keystroke("k");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        Ë‡aaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("k");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        Ë‡[FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystroke("shift-g");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        Ë‡[FOLDED]
        "
    });
    cx.simulate_keystrokes("g g");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        Ë‡[FOLDED]
        [EXCERPT]
        aaa
        bbb
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.update_editor(|editor, _, cx| {
        let buffer_ids = editor.buffer().read(cx).excerpt_buffer_ids();
        editor.fold_buffer(buffer_ids[1], cx);
    });

    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        Ë‡[FOLDED]
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
    cx.simulate_keystrokes("2 j");
    cx.assert_excerpts_with_selections(indoc! {"
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        [FOLDED]
        [EXCERPT]
        Ë‡[FOLDED]
        [EXCERPT]
        [FOLDED]
        "
    });
}

#[gpui::test]
async fn test_delete_paragraph_motion(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;
    cx.set_shared_state(indoc! {
        "Ë‡hello world.

        hello world.
        "
    })
    .await;
    cx.simulate_shared_keystrokes("y }").await;
    cx.shared_clipboard().await.assert_eq("hello world.\n");
    cx.simulate_shared_keystrokes("d }").await;
    cx.shared_state().await.assert_eq("Ë‡\nhello world.\n");
    cx.shared_clipboard().await.assert_eq("hello world.\n");

    cx.set_shared_state(indoc! {
        "helË‡lo world.

            hello world.
            "
    })
    .await;
    cx.simulate_shared_keystrokes("y }").await;
    cx.shared_clipboard().await.assert_eq("lo world.");
    cx.simulate_shared_keystrokes("d }").await;
    cx.shared_state().await.assert_eq("heË‡l\n\nhello world.\n");
    cx.shared_clipboard().await.assert_eq("lo world.");
}

#[gpui::test]
async fn test_delete_unmatched_brace(cx: &mut gpui::TestAppContext) {
    let mut cx = NeovimBackedTestContext::new(cx).await;
    cx.set_shared_state(indoc! {
        "fn o(wow: i32) {
          othË‡(wow)
          oth(wow)
        }
        "
    })
    .await;
    cx.simulate_shared_keystrokes("d ] }").await;
    cx.shared_state().await.assert_eq(indoc! {
        "fn o(wow: i32) {
          otË‡h
        }
        "
    });
    cx.shared_clipboard().await.assert_eq("(wow)\n  oth(wow)");
    cx.set_shared_state(indoc! {
        "fn o(wow: i32) {
          Ë‡oth(wow)
          oth(wow)
        }
        "
    })
    .await;
    cx.simulate_shared_keystrokes("d ] }").await;
    cx.shared_state().await.assert_eq(indoc! {
        "fn o(wow: i32) {
         Ë‡}
        "
    });
    cx.shared_clipboard()
        .await
        .assert_eq("  oth(wow)\n  oth(wow)\n");
}
