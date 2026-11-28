use freyya_core::{Refusal, SaveRead, SaveReader};

use crate::helpers::{rakaly, text_save, unavailable_rakaly};

fn hand_made_text_save(game_date: &str) -> Vec<u8> {
    text_save(
        &format!("version=\"1.15.2\" game_date={game_date} name=\"Testland\""),
        "capital=1",
        "",
    )
}

#[test]
fn reading_the_same_save_twice_processes_it_once() {
    let save = hand_made_text_save("1840.3.9.6");
    let mut reader = SaveReader::new();

    let first = reader
        .read(&save, &rakaly())
        .expect("the Save is read the first time");
    let second = reader
        .read(&save, &unavailable_rakaly())
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
        .read(&hand_made_text_save("1840.3.9.6"), &rakaly())
        .expect("the first Save is read");

    let refusal = reader
        .read(&hand_made_text_save("1840.3.16.6"), &unavailable_rakaly())
        .expect_err("a different Save needs the Save reader");

    assert!(matches!(refusal, Refusal::ReaderUnavailable { .. }));
}

#[test]
fn a_refused_save_is_not_remembered() {
    let mut reader = SaveReader::new();
    let save = hand_made_text_save("1840.3.9.6");
    reader
        .read(&save, &unavailable_rakaly())
        .expect_err("the Save reader is missing");

    let read = reader
        .read(&save, &rakaly())
        .expect("the Save is read once the Save reader is available");

    assert!(matches!(read, SaveRead::New(_)));
}
