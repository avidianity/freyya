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

#[test]
fn building_under_construction_has_its_staffing_and_employees_marked_missing() {
    let snapshot = manual_save_snapshot();
    let gotland = state(&snapshot, "STATE_GOTLAND");

    let glassworks = building(gotland, 16_782_042);

    assert_eq!(glassworks.kind, "building_glassworks");
    assert_eq!(glassworks.levels.value, Some(0));
    assert_eq!(glassworks.staffing.value, None);
    assert_eq!(glassworks.employees.value, None);
}

#[test]
fn manual_save_gives_country_wide_employment_unemployment_and_vacancies() {
    let jobs = manual_save_snapshot().buildings_and_jobs.jobs;

    assert_eq!(
        jobs.salaried_workforce,
        observed(
            438_670,
            Unit::People,
            "country_manager.database.24.pop_statistics.population_salaried_workforce"
        )
    );
    assert_eq!(
        jobs.unemployed_workforce,
        observed(
            36_823,
            Unit::People,
            "country_manager.database.24.pop_statistics.population_unemployed_workforce"
        )
    );
    assert_eq!(
        jobs.subsisting_workforce,
        observed(
            378_501,
            Unit::People,
            "country_manager.database.24.pop_statistics.population_subsisting_workforce x 100000"
        )
    );
    assert_eq!(
        jobs.vacancies,
        observed(
            105,
            Unit::People,
            "country_manager.database.24.pop_statistics.vacancies_new_culture x 100000"
        )
    );
}

#[test]
fn save_without_population_statistics_has_its_jobs_marked_missing() {
    let save = "SAV01000000000000000000\n\
         meta_data={ version=\"1.15.2\" game_date=1840.3.9.6 name=\"Testland\" }\n\
         country_manager={ database={ 0=none 7={ definition=\"TST\" } } }\n\
         states={ database={ 3={ country=7 region=\"STATE_TEST\" } } }\n\
         building_manager={ database={ 0=none 1={ building=\"building_rye_farm\" levels=2 state=3 } } }\n\
         player_manager={ database={ 0={ user=\"tester\" country=7 } } }\n";

    let snapshot = read_save(save.as_bytes(), &Rakaly::on_path()).expect("the Save is read");

    let jobs = snapshot.buildings_and_jobs.jobs;
    assert_eq!(jobs.salaried_workforce.value, None);
    assert_eq!(jobs.unemployed_workforce.value, None);
    assert_eq!(jobs.subsisting_workforce.value, None);
    assert_eq!(jobs.vacancies.value, None);
    let farm = building(&snapshot.buildings_and_jobs.states[0], 1);
    assert_eq!(farm.levels.value, Some(2));
    assert_eq!(farm.production_methods.value, None);
    assert_eq!(farm.staffing.value, None);
    assert_eq!(farm.employees.value, None);
}
