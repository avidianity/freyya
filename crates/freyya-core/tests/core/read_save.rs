use freyya_core::{GameDate, Refusal, read_save};

use crate::helpers::{manual_save, rakaly, read_manual_save, text_save};

fn hand_made_text_save(version: &str, extra_meta: &str) -> Vec<u8> {
    text_save(
        &format!("version=\"{version}\" game_date=1840.3.9.6 name=\"Testland\" {extra_meta}"),
        "capital=1",
        "",
    )
}

#[test]
fn hand_made_text_save_on_checked_version_is_read() {
    let save = hand_made_text_save("1.15.2", "");

    let snapshot = read_save(&save, &rakaly()).expect("a vanilla 1.15.2 Save is read");

    assert_eq!(
        snapshot.game_date,
        GameDate {
            year: 1840,
            month: 3,
            day: 9
        }
    );
    assert_eq!(snapshot.player_country.tag, "TST");
}

#[test]
fn save_from_unchecked_game_version_is_refused() {
    let save = hand_made_text_save("1.16.0", "");

    let refusal = read_save(&save, &rakaly()).expect_err("an unchecked version is refused");

    assert_eq!(
        refusal,
        Refusal::UnsupportedVersion {
            version: "1.16.0".to_owned()
        }
    );
}

#[test]
fn save_with_active_mods_is_refused_and_names_them() {
    let save = hand_made_text_save("1.15.2", "mods={ \"Better Economy\" \"More Goods\" }");

    let refusal = read_save(&save, &rakaly()).expect_err("a modded Save is refused");

    assert_eq!(
        refusal,
        Refusal::Modded {
            mods: vec!["Better Economy".to_owned(), "More Goods".to_owned()]
        }
    );
}

#[test]
fn manual_save_gives_version_date_and_player_country() {
    let snapshot = read_manual_save();

    assert_eq!(snapshot.game_version, "1.15.2");
    assert_eq!(
        snapshot.game_date,
        GameDate {
            year: 1837,
            month: 8,
            day: 16
        }
    );
    assert_eq!(snapshot.player_country.tag, "SWE");
    assert_eq!(snapshot.player_country.name, "Sweden");
}

#[test]
fn truncated_save_is_refused_as_incomplete() {
    let save = manual_save();
    let first_half = &save[..save.len() / 2];

    let refusal = read_save(first_half, &rakaly()).expect_err("a cut Save is refused");

    assert_eq!(refusal, Refusal::Incomplete);
}

#[test]
fn save_with_overwritten_bytes_is_refused_as_damaged() {
    let mut save = manual_save();
    let middle = save.len() / 2;
    save[middle..middle + 8].fill(0);

    let refusal = read_save(&save, &rakaly()).expect_err("a damaged Save is refused");

    assert!(
        matches!(refusal, Refusal::Damaged { .. }),
        "got {refusal:?}"
    );
}

#[test]
fn file_that_is_not_a_save_is_refused_as_damaged() {
    let refusal = read_save(b"not a save at all", &rakaly())
        .expect_err("a file that is not a Save is refused");

    assert!(
        matches!(refusal, Refusal::Damaged { .. }),
        "got {refusal:?}"
    );
}
