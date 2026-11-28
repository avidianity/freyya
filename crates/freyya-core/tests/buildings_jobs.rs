use freyya_core::buildings_jobs::{Building, Observation, StateBuildings, Unit};
use freyya_core::{Rakaly, Snapshot, read_save};

fn manual_save_snapshot() -> Snapshot {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/saves/sweden_1837_08_16.v3"
    );
    let save = std::fs::read(path).expect("the manual Save fixture is readable");
    read_save(&save, &Rakaly::on_path()).expect("the manual Save is read")
}

fn state<'a>(snapshot: &'a Snapshot, region: &str) -> &'a StateBuildings {
    snapshot
        .buildings_and_jobs
        .states
        .iter()
        .find(|state| state.region == region)
        .unwrap_or_else(|| panic!("{region} is one of the player's states"))
}

fn building(state: &StateBuildings, id: u32) -> &Building {
    state
        .buildings
        .iter()
        .find(|building| building.id == id)
        .unwrap_or_else(|| panic!("building {id} is in {}", state.region))
}

fn observed<T>(value: T, unit: Unit, source: &str) -> Observation<T> {
    Observation {
        value: Some(value),
        unit,
        source: source.to_owned(),
    }
}

#[test]
fn manual_save_gives_buildings_with_levels_methods_and_staffing() {
    let snapshot = manual_save_snapshot();
    let svealand = state(&snapshot, "STATE_SVEALAND");

    let administration = building(svealand, 0);

    assert_eq!(svealand.state_id, 133);
    assert_eq!(administration.kind, "building_government_administration");
    assert_eq!(
        administration.levels,
        observed(4, Unit::Levels, "building_manager.database.0.levels")
    );
    assert_eq!(
        administration.production_methods,
        observed(
            vec![
                "pm_religious_bureaucrats".to_owned(),
                "pm_professional_bureaucrats".to_owned(),
                "pm_horizontal_drawer_cabinets".to_owned(),
            ],
            Unit::ProductionMethods,
            "building_manager.database.0.production_methods",
        )
    );
    assert_eq!(
        administration.staffing,
        observed(
            4.0,
            Unit::StaffedLevels,
            "building_manager.database.0.staffing"
        )
    );
}

#[test]
fn manual_save_gives_employees_of_each_building_from_its_pops() {
    let snapshot = manual_save_snapshot();
    let svealand = state(&snapshot, "STATE_SVEALAND");

    assert_eq!(
        building(svealand, 0).employees,
        observed(
            6000,
            Unit::People,
            "pops.database.*.workforce where workplace=0"
        )
    );
    assert_eq!(building(svealand, 9).employees.value, Some(895));
}
