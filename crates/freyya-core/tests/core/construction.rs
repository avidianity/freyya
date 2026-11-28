use freyya_core::{Observation, QueuedConstruction, Reading, Unit};

use crate::helpers::{player_save, read_manual_save, read_text_save};

fn hand_made_text_save(country: &str, ai: &str) -> Vec<u8> {
    player_save(country, &format!("ai={{ database={{ 3=none {ai} }} }}"))
}

#[test]
fn country_with_no_queued_construction_has_empty_queues() {
    let save = hand_made_text_save("capital=1", "");

    let snapshot = read_text_save(&save);

    assert_eq!(snapshot.construction.government_queue, vec![]);
    assert_eq!(snapshot.construction.private_queue, vec![]);
}

#[test]
fn manual_save_gives_the_government_construction_queue_in_order() {
    let snapshot = read_manual_save();

    let queue = &snapshot.construction.government_queue;
    assert_eq!(queue.len(), 23);
    assert_eq!(
        queue[0],
        QueuedConstruction {
            building: "building_livestock_ranch".to_owned(),
            state: 130,
            construction_left: Observation {
                reading: Reading::Read(25.35464),
                unit: Unit::ConstructionPoints,
                source: "country_manager.database.24.government_queue.construction_elements[0].construction_left".to_owned(),
            },
            construction_speed: Observation {
                reading: Reading::Read(7.5),
                unit: Unit::ConstructionPointsPerWeek,
                source: "country_manager.database.24.government_queue.construction_elements[0].base_construction_speed".to_owned(),
            },
        }
    );
    assert_eq!(queue[1].construction_left.reading, Reading::Read(200.0));
    assert_eq!(
        queue[1].construction_speed.reading,
        Reading::Missing { reason: None }
    );
    assert_eq!(queue[2].building, "building_arms_industry");
    assert_eq!(queue[2].state, 483);
}

#[test]
fn manual_save_gives_the_construction_in_use_each_week() {
    let snapshot = read_manual_save();

    assert_eq!(
        snapshot.construction.construction_in_use,
        Observation {
            reading: Reading::Read(15.0),
            unit: Unit::ConstructionPointsPerWeek,
            source: "country_manager.database.24.government_queue and private_queue construction_elements[*].base_construction_speed (sum)".to_owned(),
        }
    );
}

#[test]
fn construction_in_use_adds_the_items_under_construction_in_both_queues() {
    let save = hand_made_text_save(
        "government_queue={ construction_elements={ \
         { type=\"building_iron_mine\" state=1 construction_left=12 base_construction_speed=4.5 } \
         { type=\"building_iron_mine\" state=1 construction_left=400 } } } \
         private_queue={ construction_elements={ \
         { type=\"building_glassworks\" state=1 construction_left=90 base_construction_speed=3 } } }",
        "",
    );

    let snapshot = read_text_save(&save);

    assert_eq!(
        snapshot.construction.construction_in_use.reading,
        Reading::Read(7.5)
    );
}

#[test]
fn construction_in_use_is_missing_when_nothing_is_under_construction() {
    let save = hand_made_text_save(
        "capital=1",
        "5={ country=7 spending_variables={ country=7 construction_output=40 } }",
    );

    let snapshot = read_text_save(&save);

    assert_eq!(
        snapshot.construction.construction_in_use.reading,
        Reading::Missing { reason: None }
    );
    assert_eq!(
        snapshot.construction.construction_in_use.unit,
        Unit::ConstructionPointsPerWeek
    );
}

#[test]
fn manual_save_gives_the_private_construction_queue() {
    let snapshot = read_manual_save();

    let queue = &snapshot.construction.private_queue;
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].building, "building_glassworks");
    assert_eq!(queue[0].state, 129);
    assert_eq!(queue[0].construction_left.reading, Reading::Read(366.76846));
    assert_eq!(queue[0].construction_speed.reading, Reading::Read(7.5));
    assert_eq!(
        queue[0].construction_left.source,
        "country_manager.database.24.private_queue.construction_elements[0].construction_left"
    );
}
