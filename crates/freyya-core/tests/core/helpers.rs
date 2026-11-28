use std::path::PathBuf;

use freyya_core::{Rakaly, Snapshot, read_save};

pub(crate) const CHECKED_META: &str = "version=\"1.15.2\" game_date=1840.3.9.6 name=\"Testland\"";

pub(crate) fn rakaly() -> Rakaly {
    Rakaly::on_path()
}

pub(crate) fn unavailable_rakaly() -> Rakaly {
    Rakaly::at("freyya-test-no-such-save-reader")
}

pub(crate) fn manual_save() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/saves/sweden_1837_08_16.v3"
    );
    std::fs::read(path).expect("the manual Save fixture is readable")
}

pub(crate) fn read_manual_save() -> Snapshot {
    read_save(&manual_save(), &rakaly()).expect("the manual Save is read")
}

pub(crate) fn read_text_save(save: &[u8]) -> Snapshot {
    read_save(save, &rakaly()).expect("the hand-made Save is read")
}

pub(crate) fn text_save(meta: &str, country: &str, sections: &str) -> Vec<u8> {
    format!(
        "SAV01000000000000000000\n\
         meta_data={{ {meta} }}\n\
         country_manager={{ database={{ 0=none 7={{ definition=\"TST\" {country} }} }} }}\n\
         {sections}\n\
         player_manager={{ database={{ 0={{ user=\"tester\" country=7 }} }} }}\n"
    )
    .into_bytes()
}

pub(crate) fn empty_temp_folder(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("freyya-{}-{name}", std::process::id()));
    if root.exists() {
        std::fs::remove_dir_all(&root).expect("an old test folder is removable");
    }
    root
}

pub(crate) fn player_save(country: &str, sections: &str) -> Vec<u8> {
    text_save(CHECKED_META, country, sections)
}
