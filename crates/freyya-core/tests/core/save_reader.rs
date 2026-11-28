use freyya_core::{Rakaly, Refusal, SaveRead, SaveReader};

fn hand_made_text_save(game_date: &str) -> Vec<u8> {
    format!(
        "SAV01000000000000000000\n\
         meta_data={{\n\
         \tversion=\"1.15.2\"\n\
         \tgame_date={game_date}\n\
         \tname=\"Testland\"\n\
         }}\n\
         country_manager={{ database={{ 0=none 7={{ definition=\"TST\" capital=1 }} }} }}\n\
         player_manager={{ database={{ 0={{ user=\"tester\" country=7 }} }} }}\n"
    )
    .into_bytes()
}

fn unavailable_reader() -> Rakaly {
    Rakaly::at("freyya-test-no-such-save-reader")
}

#[test]
fn reading_the_same_save_twice_processes_it_once() {
    let save = hand_made_text_save("1840.3.9.6");
    let mut reader = SaveReader::new();

    let first = reader
        .read(&save, &Rakaly::on_path())
        .expect("the Save is read the first time");
    let second = reader
        .read(&save, &unavailable_reader())
        .expect("the Save already read is recognised without the Save reader");

    let SaveRead::New(first) = first else {
        panic!("the first read processes the Save");
    };
    let SaveRead::AlreadyRead(second) = second else {
        panic!("the second read recognises the Save");
    };
    assert_eq!(first, second);
    assert_eq!(second.game_date.to_string(), "1840-03-09");
}

#[test]
fn a_save_with_different_content_is_processed_again() {
    let mut reader = SaveReader::new();
    reader
        .read(&hand_made_text_save("1840.3.9.6"), &Rakaly::on_path())
        .expect("the first Save is read");

    let refusal = reader
        .read(&hand_made_text_save("1840.3.16.6"), &unavailable_reader())
        .expect_err("a different Save needs the Save reader");

    assert!(matches!(refusal, Refusal::ReaderUnavailable { .. }));
}

#[test]
fn a_refused_save_is_not_remembered() {
    let mut reader = SaveReader::new();
    let save = hand_made_text_save("1840.3.9.6");
    reader
        .read(&save, &unavailable_reader())
        .expect_err("the Save reader is missing");

    let read = reader
        .read(&save, &Rakaly::on_path())
        .expect("the Save is read once the Save reader is available");

    assert!(matches!(read, SaveRead::New(_)));
}
