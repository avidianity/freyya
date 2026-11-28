use freyya_core::{GameDate, Rakaly, read_save};

fn manual_save() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/saves/sweden_1837_08_16.v3"
    );
    std::fs::read(path).expect("the manual Save fixture is readable")
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
