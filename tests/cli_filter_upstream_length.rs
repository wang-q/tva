#[macro_use]
#[path = "common/mod.rs"]
mod common;

use common::TvaCmd;

#[test]
fn upstream_char_len_eq_3_0() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-eq",
            "3:0",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "100\t100\t\tAbC\n", "100\t101\t\t\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_eq_3_1() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-eq",
            "3:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "0\t0.0\tz\tAzB\n",
        "-2\t-2.0\tß\tss\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_eq_3_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-eq",
            "3:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_ne_3_0() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-ne",
            "3:0",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\tabc\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_ne_3_1() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-ne",
            "3:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_ne_3_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-ne",
            "3:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_le_4_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-le",
            "4:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "-2\t-2.0\tß\tss\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_lt_4_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-lt",
            "4:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_gt_4_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-gt",
            "4:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_ge_4_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-ge",
            "4:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_le_3_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-le",
            "3:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "Mixed1\ta-雪\ta\tabcd\n",
        "Mixed2\tab-雪雪\tab\tabc\n",
        "Mixed3\tabc-雪\tabc\tab\n",
        "Mixed6\tab-雪\t雪\tabc\n",
        "Mixed7\tabc-雪\t雪雪\tab\n",
        "Mixed8\tabcd-雪\t雪雪雪\ta\n",
        "Mixed11\tabc-雪\tषि\tab\n",
        "Mixed12\tabcd-雪\tषिषि\ta\n",
        "Mixed13\ta-雪\tषिषिषि\tabcd\n",
        "Mixed16\tabcd-雪\taषि雪\ta\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_lt_3_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-lt",
            "3:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "Mixed1\ta-雪\ta\tabcd\n",
        "Mixed2\tab-雪雪\tab\tabc\n",
        "Mixed6\tab-雪\t雪\tabc\n",
        "Mixed7\tabc-雪\t雪雪\tab\n",
        "Mixed11\tabc-雪\tषि\tab\n",
        "Mixed12\tabcd-雪\tषिषि\ta\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_ge_3_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-ge",
            "3:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "English\tsnow storm\tsoccer player\ttown hall\n",
        "Chinese (Simplified)\t雪风暴\t足球运动员\t市政厅\n",
        "Chinese (Traditional)\t雪風暴\t足球運動員\t市政廳\n",
        "French\tTempête de neige\tjoueur de foot\tmairie\n",
        "Georgian\tთოვლის ქარიშხალი\tფეხბურთის მოთამაშე\tმუნიციპალიტეტი\n",
        "German\tSchneesturm\tFußballspieler\tRathaus\n",
        "Greek\tΧιονοθύελλα\tποδοσφαιριστής\tΔημαρχείο\n",
        "Japanese\t吹雪\tサッカー選手\t町役場\n",
        "Russian\tСнежная буря\tфутболист\tратуша\n",
        "Spanish\tTormenta de nieve\tjugador de fútbol\tAyuntamiento\n",
        "Vietnamese\tBão tuyết\tcầuthủ bóng đá\tThị trấn\n",
        "Mixed3\tabc-雪\tabc\tab\n",
        "Mixed4\tabcd-雪雪\tabcd\ta\n",
        "Mixed5\ta-雪\tabcde\tabcd\n",
        "Mixed8\tabcd-雪\t雪雪雪\ta\n",
        "Mixed9\ta-雪\t雪雪雪雪\tabcd\n",
        "Mixed10\tab-雪\t雪雪雪雪雪\tabc\n",
        "Mixed13\ta-雪\tषिषिषि\tabcd\n",
        "Mixed14\tab-雪\tषिषिषिषि\tabc\n",
        "Mixed15\tabc-雪\tषिषिषिषिषि\tab\n",
        "Mixed16\tabcd-雪\taषि雪\ta\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_gt_3_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-gt",
            "3:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "English\tsnow storm\tsoccer player\ttown hall\n",
        "Chinese (Simplified)\t雪风暴\t足球运动员\t市政厅\n",
        "Chinese (Traditional)\t雪風暴\t足球運動員\t市政廳\n",
        "French\tTempête de neige\tjoueur de foot\tmairie\n",
        "Georgian\tთოვლის ქარიშხალი\tფეხბურთის მოთამაშე\tმუნიციპალიტეტი\n",
        "German\tSchneesturm\tFußballspieler\tRathaus\n",
        "Greek\tΧιονοθύελλα\tποδοσφαιριστής\tΔημαρχείο\n",
        "Japanese\t吹雪\tサッカー選手\t町役場\n",
        "Russian\tСнежная буря\tфутболист\tратуша\n",
        "Spanish\tTormenta de nieve\tjugador de fútbol\tAyuntamiento\n",
        "Vietnamese\tBão tuyết\tcầuthủ bóng đá\tThị trấn\n",
        "Mixed4\tabcd-雪雪\tabcd\ta\n",
        "Mixed5\ta-雪\tabcde\tabcd\n",
        "Mixed9\ta-雪\t雪雪雪雪\tabcd\n",
        "Mixed10\tab-雪\t雪雪雪雪雪\tabc\n",
        "Mixed14\tab-雪\tषिषिषिषि\tabc\n",
        "Mixed15\tabc-雪\tषिषिषिषिषि\tab\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_eq_3_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-eq",
            "3:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "Mixed3\tabc-雪\tabc\tab\n",
        "Mixed8\tabcd-雪\t雪雪雪\ta\n",
        "Mixed13\ta-雪\tषिषिषि\tabcd\n",
        "Mixed16\tabcd-雪\taषि雪\ta\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_ne_3_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-ne",
            "3:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "English\tsnow storm\tsoccer player\ttown hall\n",
        "Chinese (Simplified)\t雪风暴\t足球运动员\t市政厅\n",
        "Chinese (Traditional)\t雪風暴\t足球運動員\t市政廳\n",
        "French\tTempête de neige\tjoueur de foot\tmairie\n",
        "Georgian\tთოვლის ქარიშხალი\tფეხბურთის მოთამაშე\tმუნიციპალიტეტი\n",
        "German\tSchneesturm\tFußballspieler\tRathaus\n",
        "Greek\tΧιονοθύελλα\tποδοσφαιριστής\tΔημαρχείο\n",
        "Japanese\t吹雪\tサッカー選手\t町役場\n",
        "Russian\tСнежная буря\tфутболист\tратуша\n",
        "Spanish\tTormenta de nieve\tjugador de fútbol\tAyuntamiento\n",
        "Vietnamese\tBão tuyết\tcầuthủ bóng đá\tThị trấn\n",
        "Mixed1\ta-雪\ta\tabcd\n",
        "Mixed2\tab-雪雪\tab\tabc\n",
        "Mixed4\tabcd-雪雪\tabcd\ta\n",
        "Mixed5\ta-雪\tabcde\tabcd\n",
        "Mixed6\tab-雪\t雪\tabc\n",
        "Mixed7\tabc-雪\t雪雪\tab\n",
        "Mixed9\ta-雪\t雪雪雪雪\tabcd\n",
        "Mixed10\tab-雪\t雪雪雪雪雪\tabc\n",
        "Mixed11\tabc-雪\tषि\tab\n",
        "Mixed12\tabcd-雪\tषिषि\ta\n",
        "Mixed14\tab-雪\tषिषिषिषि\tabc\n",
        "Mixed15\tabc-雪\tषिषिषिषिषि\tab\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_lt_1_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-lt",
            "1:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!("Language\tText 1\tText 2\tText 3\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_le_2_2_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-le",
            "2:2",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "Japanese\t吹雪\tサッカー選手\t町役場\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_ge_4_2_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-ge",
            "4:2",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "English\tsnow storm\tsoccer player\ttown hall\n",
        "Chinese (Simplified)\t雪风暴\t足球运动员\t市政厅\n",
        "Chinese (Traditional)\t雪風暴\t足球運動員\t市政廳\n",
        "French\tTempête de neige\tjoueur de foot\tmairie\n",
        "Georgian\tთოვლის ქარიშხალი\tფეხბურთის მოთამაშე\tმუნიციპალიტეტი\n",
        "German\tSchneesturm\tFußballspieler\tRathaus\n",
        "Greek\tΧιονοθύελλα\tποδοσφαιριστής\tΔημαρχείο\n",
        "Japanese\t吹雪\tサッカー選手\t町役場\n",
        "Russian\tСнежная буря\tфутболист\tратуша\n",
        "Spanish\tTormenta de nieve\tjugador de fútbol\tAyuntamiento\n",
        "Vietnamese\tBão tuyết\tcầuthủ bóng đá\tThị trấn\n",
        "Mixed1\ta-雪\ta\tabcd\n",
        "Mixed2\tab-雪雪\tab\tabc\n",
        "Mixed3\tabc-雪\tabc\tab\n",
        "Mixed5\ta-雪\tabcde\tabcd\n",
        "Mixed6\tab-雪\t雪\tabc\n",
        "Mixed7\tabc-雪\t雪雪\tab\n",
        "Mixed9\ta-雪\t雪雪雪雪\tabcd\n",
        "Mixed10\tab-雪\t雪雪雪雪雪\tabc\n",
        "Mixed11\tabc-雪\tषि\tab\n",
        "Mixed13\ta-雪\tषिषिषि\tabcd\n",
        "Mixed14\tab-雪\tषिषिषिषि\tabc\n",
        "Mixed15\tabc-雪\tषिषिषिषिषि\tab\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_ge_text_star_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-ge",
            "Text*:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "English\tsnow storm\tsoccer player\ttown hall\n",
        "Chinese (Simplified)\t雪风暴\t足球运动员\t市政厅\n",
        "Chinese (Traditional)\t雪風暴\t足球運動員\t市政廳\n",
        "French\tTempête de neige\tjoueur de foot\tmairie\n",
        "Georgian\tთოვლის ქარიშხალი\tფეხბურთის მოთამაშე\tმუნიციპალიტეტი\n",
        "German\tSchneesturm\tFußballspieler\tRathaus\n",
        "Greek\tΧιονοθύελλα\tποδοσφαιριστής\tΔημαρχείο\n",
        "Russian\tСнежная буря\tфутболист\tратуша\n",
        "Spanish\tTormenta de nieve\tjugador de fútbol\tAyuntamiento\n",
        "Vietnamese\tBão tuyết\tcầuthủ bóng đá\tThị trấn\n",
        "Mixed5\ta-雪\tabcde\tabcd\n",
        "Mixed9\ta-雪\t雪雪雪雪\tabcd\n",
        "Mixed10\tab-雪\t雪雪雪雪雪\tabc\n",
        "Mixed13\ta-雪\tषिषिषि\tabcd\n",
        "Mixed14\tab-雪\tषिषिषिषि\tabc\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_ge_text_star_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-ge",
            "Text*:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "English\tsnow storm\tsoccer player\ttown hall\n",
        "Chinese (Simplified)\t雪风暴\t足球运动员\t市政厅\n",
        "Chinese (Traditional)\t雪風暴\t足球運動員\t市政廳\n",
        "French\tTempête de neige\tjoueur de foot\tmairie\n",
        "Georgian\tთოვლის ქარიშხალი\tფეხბურთის მოთამაშე\tმუნიციპალიტეტი\n",
        "German\tSchneesturm\tFußballspieler\tRathaus\n",
        "Greek\tΧιονοθύελλα\tποδοσφαιριστής\tΔημαρχείο\n",
        "Japanese\t吹雪\tサッカー選手\t町役場\n",
        "Russian\tСнежная буря\tфутболист\tратуша\n",
        "Spanish\tTormenta de nieve\tjugador de fútbol\tAyuntamiento\n",
        "Vietnamese\tBão tuyết\tcầuthủ bóng đá\tThị trấn\n",
        "Mixed5\ta-雪\tabcde\tabcd\n",
        "Mixed6\tab-雪\t雪\tabc\n",
        "Mixed9\ta-雪\t雪雪雪雪\tabcd\n",
        "Mixed10\tab-雪\t雪雪雪雪雪\tabc\n",
        "Mixed13\ta-雪\tषिषिषि\tabcd\n",
        "Mixed14\tab-雪\tषिषिषिषि\tabc\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_lt_header_escaped_space() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-lt",
            "Text\\ 2:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "Mixed1\ta-雪\ta\tabcd\n",
        "Mixed2\tab-雪雪\tab\tabc\n",
        "Mixed6\tab-雪\t雪\tabc\n",
        "Mixed7\tabc-雪\t雪雪\tab\n",
        "Mixed11\tabc-雪\tषि\tab\n",
        "Mixed12\tabcd-雪\tषिषि\ta\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_lt_header_escaped_space_arg_sep() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-lt",
            "Text\\ 2 3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "Mixed1\ta-雪\ta\tabcd\n",
        "Mixed2\tab-雪雪\tab\tabc\n",
        "Mixed6\tab-雪\t雪\tabc\n",
        "Mixed7\tabc-雪\t雪雪\tab\n",
        "Mixed11\tabc-雪\tषि\tab\n",
        "Mixed12\tabcd-雪\tषिषि\ta\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_ge_range_input4() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-ge",
            "2-7:2",
            "tests/data/filter/input4.tsv",
        ])
        .run();
    let expected = concat!(
        "line\t2_apha\t3_apha\t4_num\t5_num\t6_num\t7_alpha\t8_num\t9_num\n",
        "1\tabc\tdef\t10\t20\t30\tghi\t40\t50\n",
        "3\tcde\tde\t35\t45\t55\tbcdef\t10\t25\n",
        "4\taadd\taabdd\t10\t30\t15\tabd\t25\t25\n",
        "11\tAADD\tAABDD\t10\t30\t15\tABD\t25\t25\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_eq_3_0() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-eq",
            "3:0",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "100\t100\t\tAbC\n", "100\t101\t\t\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_eq_3_1() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-eq",
            "3:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "0\t0.0\tz\tAzB\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_eq_3_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-eq",
            "3:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!("F1\tF2\tF3\tF4\n", "-2\t-2.0\tß\tss\n",);
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_ne_3_0() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-ne",
            "3:0",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\tabc\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_ne_3_1() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-ne",
            "3:1",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_ne_3_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-ne",
            "3:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_le_4_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-le",
            "4:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "-2\t-2.0\tß\tss\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_lt_4_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-lt",
            "4:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "1\t1.0\ta\tA\n",
        "2\t2.\tb\tB\n",
        "100\t100\tabc\t\n",
        "100\t101\t\t\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_gt_4_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-gt",
            "4:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_ge_4_2() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-ge",
            "4:2",
            "tests/data/filter/input1.tsv",
        ])
        .run();
    let expected = concat!(
        "F1\tF2\tF3\tF4\n",
        "10\t10.1\tabc\tABC\n",
        "100\t100\tabc\tAbC\n",
        "0\t0.0\tz\tAzB\n",
        "-1\t-0.1\tabc def\tabc def\n",
        "-2\t-2.0\tß\tss\n",
        "0.\t100.\tàbc\tÀBC\n",
        "0.0\t100.0\tàßc\tÀssC\n",
        "-0.0\t-100.0\tàßc\tÀSSC\n",
        "100\t100\t\tAbC\n",
        "100\t102\tabc\tAbC\n",
        "100\t103\tabc\tAbC\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_le_3_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-le",
            "3:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "Mixed1\ta-雪\ta\tabcd\n",
        "Mixed2\tab-雪雪\tab\tabc\n",
        "Mixed3\tabc-雪\tabc\tab\n",
        "Mixed6\tab-雪\t雪\tabc\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_lt_3_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-lt",
            "3:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "Mixed1\ta-雪\ta\tabcd\n",
        "Mixed2\tab-雪雪\tab\tabc\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_ge_3_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-ge",
            "3:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "English\tsnow storm\tsoccer player\ttown hall\n",
        "Chinese (Simplified)\t雪风暴\t足球运动员\t市政厅\n",
        "Chinese (Traditional)\t雪風暴\t足球運動員\t市政廳\n",
        "French\tTempête de neige\tjoueur de foot\tmairie\n",
        "Georgian\tთოვლის ქარიშხალი\tფეხბურთის მოთამაშე\tმუნიციპალიტეტი\n",
        "German\tSchneesturm\tFußballspieler\tRathaus\n",
        "Greek\tΧιονοθύελλα\tποδοσφαιριστής\tΔημαρχείο\n",
        "Japanese\t吹雪\tサッカー選手\t町役場\n",
        "Russian\tСнежная буря\tфутболист\tратуша\n",
        "Spanish\tTormenta de nieve\tjugador de fútbol\tAyuntamiento\n",
        "Vietnamese\tBão tuyết\tcầuthủ bóng đá\tThị trấn\n",
        "Mixed3\tabc-雪\tabc\tab\n",
        "Mixed4\tabcd-雪雪\tabcd\ta\n",
        "Mixed5\ta-雪\tabcde\tabcd\n",
        "Mixed6\tab-雪\t雪\tabc\n",
        "Mixed7\tabc-雪\t雪雪\tab\n",
        "Mixed8\tabcd-雪\t雪雪雪\ta\n",
        "Mixed9\ta-雪\t雪雪雪雪\tabcd\n",
        "Mixed10\tab-雪\t雪雪雪雪雪\tabc\n",
        "Mixed11\tabc-雪\tषि\tab\n",
        "Mixed12\tabcd-雪\tषिषि\ta\n",
        "Mixed13\ta-雪\tषिषिषि\tabcd\n",
        "Mixed14\tab-雪\tषिषिषिषि\tabc\n",
        "Mixed15\tabc-雪\tषिषिषिषिषि\tab\n",
        "Mixed16\tabcd-雪\taषि雪\ta\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_gt_3_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-gt",
            "3:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "English\tsnow storm\tsoccer player\ttown hall\n",
        "Chinese (Simplified)\t雪风暴\t足球运动员\t市政厅\n",
        "Chinese (Traditional)\t雪風暴\t足球運動員\t市政廳\n",
        "French\tTempête de neige\tjoueur de foot\tmairie\n",
        "Georgian\tთოვლის ქარიშხალი\tფეხბურთის მოთამაშე\tმუნიციპალიტეტი\n",
        "German\tSchneesturm\tFußballspieler\tRathaus\n",
        "Greek\tΧιονοθύελλα\tποδοσφαιριστής\tΔημαρχείο\n",
        "Japanese\t吹雪\tサッカー選手\t町役場\n",
        "Russian\tСнежная буря\tфутболист\tратуша\n",
        "Spanish\tTormenta de nieve\tjugador de fútbol\tAyuntamiento\n",
        "Vietnamese\tBão tuyết\tcầuthủ bóng đá\tThị trấn\n",
        "Mixed4\tabcd-雪雪\tabcd\ta\n",
        "Mixed5\ta-雪\tabcde\tabcd\n",
        "Mixed7\tabc-雪\t雪雪\tab\n",
        "Mixed8\tabcd-雪\t雪雪雪\ta\n",
        "Mixed9\ta-雪\t雪雪雪雪\tabcd\n",
        "Mixed10\tab-雪\t雪雪雪雪雪\tabc\n",
        "Mixed11\tabc-雪\tषि\tab\n",
        "Mixed12\tabcd-雪\tषिषि\ta\n",
        "Mixed13\ta-雪\tषिषिषि\tabcd\n",
        "Mixed14\tab-雪\tषिषिषिषि\tabc\n",
        "Mixed15\tabc-雪\tषिषिषिषिषि\tab\n",
        "Mixed16\tabcd-雪\taषि雪\ta\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_eq_3_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-eq",
            "3:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "Mixed3\tabc-雪\tabc\tab\n",
        "Mixed6\tab-雪\t雪\tabc\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_ne_3_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-ne",
            "3:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "English\tsnow storm\tsoccer player\ttown hall\n",
        "Chinese (Simplified)\t雪风暴\t足球运动员\t市政厅\n",
        "Chinese (Traditional)\t雪風暴\t足球運動員\t市政廳\n",
        "French\tTempête de neige\tjoueur de foot\tmairie\n",
        "Georgian\tთოვლის ქარიშხალი\tფეხბურთის მოთამაშე\tმუნიციპალიტეტი\n",
        "German\tSchneesturm\tFußballspieler\tRathaus\n",
        "Greek\tΧιονοθύελλα\tποδοσφαιριστής\tΔημαρχείο\n",
        "Japanese\t吹雪\tサッカー選手\t町役場\n",
        "Russian\tСнежная буря\tфутболист\tратуша\n",
        "Spanish\tTormenta de nieve\tjugador de fútbol\tAyuntamiento\n",
        "Vietnamese\tBão tuyết\tcầuthủ bóng đá\tThị trấn\n",
        "Mixed1\ta-雪\ta\tabcd\n",
        "Mixed2\tab-雪雪\tab\tabc\n",
        "Mixed4\tabcd-雪雪\tabcd\ta\n",
        "Mixed5\ta-雪\tabcde\tabcd\n",
        "Mixed7\tabc-雪\t雪雪\tab\n",
        "Mixed8\tabcd-雪\t雪雪雪\ta\n",
        "Mixed9\ta-雪\t雪雪雪雪\tabcd\n",
        "Mixed10\tab-雪\t雪雪雪雪雪\tabc\n",
        "Mixed11\tabc-雪\tषि\tab\n",
        "Mixed12\tabcd-雪\tषिषि\ta\n",
        "Mixed13\ta-雪\tषिषिषि\tabcd\n",
        "Mixed14\tab-雪\tषिषिषिषि\tabc\n",
        "Mixed15\tabc-雪\tषिषिषिषिषि\tab\n",
        "Mixed16\tabcd-雪\taषि雪\ta\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_ge_text_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-ge",
            "Text*:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "English\tsnow storm\tsoccer player\ttown hall\n",
        "Chinese (Simplified)\t雪风暴\t足球运动员\t市政厅\n",
        "Chinese (Traditional)\t雪風暴\t足球運動員\t市政廳\n",
        "French\tTempête de neige\tjoueur de foot\tmairie\n",
        "Georgian\tთოვლის ქარიშხალი\tფეხბურთის მოთამაშე\tმუნიციპალიტეტი\n",
        "German\tSchneesturm\tFußballspieler\tRathaus\n",
        "Greek\tΧιονοθύελλα\tποδοσφαιριστής\tΔημαρχείο\n",
        "Russian\tСнежная буря\tфутболист\tратуша\n",
        "Spanish\tTormenta de nieve\tjugador de fútbol\tAyuntamiento\n",
        "Vietnamese\tBão tuyết\tcầuthủ bóng đá\tThị trấn\n",
        "Mixed5\ta-雪\tabcde\tabcd\n",
        "Mixed9\ta-雪\t雪雪雪雪\tabcd\n",
        "Mixed10\tab-雪\t雪雪雪雪雪\tabc\n",
        "Mixed13\ta-雪\tषिषिषि\tabcd\n",
        "Mixed14\tab-雪\tषिषिषिषि\tabc\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_byte_len_ge_text_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--byte-len-ge",
            "Text*:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "English\tsnow storm\tsoccer player\ttown hall\n",
        "Chinese (Simplified)\t雪风暴\t足球运动员\t市政厅\n",
        "Chinese (Traditional)\t雪風暴\t足球運動員\t市政廳\n",
        "French\tTempête de neige\tjoueur de foot\tmairie\n",
        "Georgian\tთოვლის ქარიშხალი\tფეხბურთის მოთამაშე\tმუნიციპალიტეტი\n",
        "German\tSchneesturm\tFußballspieler\tRathaus\n",
        "Greek\tΧιονοθύελλα\tποδοσφαιριστής\tΔημαρχείο\n",
        "Japanese\t吹雪\tサッカー選手\t町役場\n",
        "Russian\tСнежная буря\tфутболист\tратуша\n",
        "Spanish\tTormenta de nieve\tjugador de fútbol\tAyuntamiento\n",
        "Vietnamese\tBão tuyết\tcầuthủ bóng đá\tThị trấn\n",
        "Mixed5\ta-雪\tabcde\tabcd\n",
        "Mixed6\tab-雪\t雪\tabc\n",
        "Mixed9\ta-雪\t雪雪雪雪\tabcd\n",
        "Mixed10\tab-雪\t雪雪雪雪雪\tabc\n",
        "Mixed13\ta-雪\tषिषिषि\tabcd\n",
        "Mixed14\tab-雪\tषिषिषिषि\tabc\n",
    );
    assert_eq!(stdout, expected);
}

#[test]
fn upstream_char_len_lt_text_2_3_unicode() {
    let (stdout, _) = TvaCmd::new()
        .args(&[
            "filter",
            "--header",
            "--char-len-lt",
            "Text 2:3",
            "tests/data/filter/input_unicode.tsv",
        ])
        .run();
    let expected = concat!(
        "Language\tText 1\tText 2\tText 3\n",
        "Mixed1\ta-雪\ta\tabcd\n",
        "Mixed2\tab-雪雪\tab\tabc\n",
        "Mixed6\tab-雪\t雪\tabc\n",
        "Mixed7\tabc-雪\t雪雪\tab\n",
        "Mixed11\tabc-雪\tषि\tab\n",
        "Mixed12\tabcd-雪\tषिषि\ta\n",
    );
    assert_eq!(stdout, expected);
}
