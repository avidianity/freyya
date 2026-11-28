use freyya_core::{Observation, QueuedConstruction, Rakaly, Unit, read_save};

fn manual_save() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/saves/sweden_1837_08_16.v3"
    );
    std::fs::read(path).expect("the manual Save fixture is readable")
}

fn hand_made_text_save(country: &str, ai: &str) -> Vec<u8> {
    format!(
        "SAV01000000000000000000\n\
         meta_data={{ version=\"1.15.2\" game_date=1840.3.9.6 name=\"Testland\" }}\n\
         country_manager={{ database={{ 0=none 7={{ definition=\"TST\" {country} }} }} }}\n\
         ai={{ database={{ 3=none {ai} }} }}\n\
         player_manager={{ database={{ 0={{ user=\"tester\" country=7 }} }} }}\n"
    )
    .into_bytes()
}

#[test]
fn country_with_no_queued_construction_has_empty_queues() {
    let save = hand_made_text_save("capital=1", "");

    let snapshot = read_save(&save, &Rakaly::on_path()).expect("the hand-made Save is read");

    assert_eq!(snapshot.construction.government_queue, vec![]);
    assert_eq!(snapshot.construction.private_queue, vec![]);
}

#[test]
fn manual_save_gives_the_government_construction_queue_in_order() {
    let snapshot = read_save(&manual_save(), &Rakaly::on_path()).expect("the manual Save is read");

    let queue = &snapshot.construction.government_queue;
    assert_eq!(queue.len(), 23);
    assert_eq!(
        queue[0],
        QueuedConstruction {
            building: "building_livestock_ranch".to_owned(),
            state: 130,
            construction_left: Observation {
                value: Some(25.35464),
                unit: Unit::ConstructionPoints,
                source: "country_manager.database.24.government_queue.construction_elements[0].construction_left".to_owned(),
            },
            construction_speed: Observation {
                value: Some(7.77225),
                unit: Unit::ConstructionPointsPerWeek,
                source: "country_manager.database.24.government_queue.construction_elements[0].construction_speed".to_owned(),
            },
        }
    );
    assert_eq!(queue[1].construction_left.value, Some(200.0));
    assert_eq!(queue[1].construction_speed.value, None);
    assert_eq!(queue[2].building, "building_arms_industry");
    assert_eq!(queue[2].state, 483);
}

#[test]
fn manual_save_gives_the_construction_capacity() {
    let snapshot = read_save(&manual_save(), &Rakaly::on_path()).expect("the manual Save is read");

    assert_eq!(
        snapshot.construction.capacity,
        Observation {
            value: Some(25.0),
            unit: Unit::ConstructionPointsPerWeek,
            source: "ai.database.326.spending_variables.construction_output".to_owned(),
        }
    );
}

#[test]
fn construction_capacity_is_missing_when_the_save_does_not_record_it() {
    let save = hand_made_text_save(
        "capital=1",
        "5={ country=9 spending_variables={ country=9 construction_output=40 } }",
    );

    let snapshot = read_save(&save, &Rakaly::on_path()).expect("the hand-made Save is read");

    assert_eq!(snapshot.construction.capacity.value, None);
    assert_eq!(
        snapshot.construction.capacity.unit,
        Unit::ConstructionPointsPerWeek
    );
}

#[test]
fn manual_save_gives_the_private_construction_queue() {
    let snapshot = read_save(&manual_save(), &Rakaly::on_path()).expect("the manual Save is read");

    let queue = &snapshot.construction.private_queue;
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].building, "building_glassworks");
    assert_eq!(queue[0].state, 129);
    assert_eq!(queue[0].construction_left.value, Some(366.76846));
    assert_eq!(queue[0].construction_speed.value, Some(7.77225));
    assert_eq!(
        queue[0].construction_left.source,
        "country_manager.database.24.private_queue.construction_elements[0].construction_left"
    );
}
