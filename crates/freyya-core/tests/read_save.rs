use freyya_core::{GameDate, Rakaly, Refusal, read_save};

fn manual_save() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/saves/sweden_1837_08_16.v3"
    );
    std::fs::read(path).expect("the manual Save fixture is readable")
}

fn hand_made_text_save(version: &str, extra_meta: &str) -> Vec<u8> {
    format!(
        "SAV01000000000000000000\n\
         meta_data={{\n\
         \tversion=\"{version}\"\n\
         \tgame_date=1840.3.9.6\n\
         \tname=\"Testland\"\n\
         {extra_meta}\n\
         }}\n\
         country_manager={{ database={{ 0=none 7={{ definition=\"TST\" capital=1 }} }} }}\n\
         player_manager={{ database={{ 0={{ user=\"tester\" country=7 }} }} }}\n"
    )
    .into_bytes()
}

#[test]
fn hand_made_text_save_on_checked_version_is_read() {
    let save = hand_made_text_save("1.15.2", "");

    let snapshot = read_save(&save, &Rakaly::on_path()).expect("a vanilla 1.15.2 Save is read");

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

    let refusal =
        read_save(&save, &Rakaly::on_path()).expect_err("an unchecked version is refused");

    assert_eq!(
        refusal,
        Refusal::UnsupportedVersion {
            version: "1.16.0".to_owned()
        }
    );
}

#[test]
fn save_with_active_mods_is_refused_and_names_them() {
    let save = hand_made_text_save("1.15.2", "\tmods={ \"Better Economy\" \"More Goods\" }");

    let refusal = read_save(&save, &Rakaly::on_path()).expect_err("a modded Save is refused");

    assert_eq!(
        refusal,
        Refusal::Modded {
            mods: vec!["Better Economy".to_owned(), "More Goods".to_owned()]
        }
    );
}

#[test]
fn manual_save_gives_version_date_and_player_country() {
    let snapshot = read_save(&manual_save(), &Rakaly::on_path()).expect("the manual Save is read");

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

    let refusal = read_save(first_half, &Rakaly::on_path()).expect_err("a cut Save is refused");

    assert_eq!(refusal, Refusal::Incomplete);
}

#[test]
fn save_with_overwritten_bytes_is_refused_as_damaged() {
    let mut save = manual_save();
    let middle = save.len() / 2;
    save[middle..middle + 8].fill(0);

    let refusal = read_save(&save, &Rakaly::on_path()).expect_err("a damaged Save is refused");

    assert!(
        matches!(refusal, Refusal::Damaged { .. }),
        "got {refusal:?}"
    );
}

#[test]
fn file_that_is_not_a_save_is_refused_as_damaged() {
    let refusal = read_save(b"not a save at all", &Rakaly::on_path())
        .expect_err("a file that is not a Save is refused");

    assert!(
        matches!(refusal, Refusal::Damaged { .. }),
        "got {refusal:?}"
    );
}
