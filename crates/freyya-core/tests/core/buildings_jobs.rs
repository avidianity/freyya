use freyya_core::Snapshot;
use freyya_core::buildings_jobs::{Building, StateBuildings};
use freyya_core::{Observation, Reading, Unit};

use crate::helpers::{player_save, read_manual_save, read_text_save};

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
        reading: Reading::Read(value),
        unit,
        source: source.to_owned(),
    }
}

#[test]
fn manual_save_gives_buildings_with_levels_methods_and_staffing() {
    let snapshot = read_manual_save();
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
    let snapshot = read_manual_save();
    let svealand = state(&snapshot, "STATE_SVEALAND");

    assert_eq!(
        building(svealand, 0).employees,
        observed(
            6000,
            Unit::People,
            "pops.database.*.workforce where workplace=0"
        )
    );
    assert_eq!(building(svealand, 9).employees.reading, Reading::Read(895));
}

#[test]
fn building_under_construction_has_its_staffing_and_employees_marked_missing() {
    let snapshot = read_manual_save();
    let gotland = state(&snapshot, "STATE_GOTLAND");

    let glassworks = building(gotland, 16_782_042);

    assert_eq!(glassworks.kind, "building_glassworks");
    assert_eq!(glassworks.levels.reading, Reading::Read(0));
    assert_eq!(
        glassworks.staffing.reading,
        Reading::Missing { reason: None }
    );
    assert_eq!(
        glassworks.employees.reading,
        Reading::Missing { reason: None }
    );
}

#[test]
fn manual_save_gives_country_wide_employment_unemployment_and_vacancies() {
    let jobs = read_manual_save().buildings_and_jobs.jobs;

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
    let save = player_save(
        "",
        "states={ database={ 3={ country=7 region=\"STATE_TEST\" } } }\n\
         building_manager={ database={ 0=none 1={ building=\"building_rye_farm\" levels=2 state=3 } } }",
    );

    let snapshot = read_text_save(&save);

    let jobs = snapshot.buildings_and_jobs.jobs;
    assert_eq!(
        jobs.salaried_workforce.reading,
        Reading::Missing { reason: None }
    );
    assert_eq!(
        jobs.unemployed_workforce.reading,
        Reading::Missing { reason: None }
    );
    assert_eq!(
        jobs.subsisting_workforce.reading,
        Reading::Missing { reason: None }
    );
    assert_eq!(jobs.vacancies.reading, Reading::Missing { reason: None });
    let farm = building(&snapshot.buildings_and_jobs.states[0], 1);
    assert_eq!(farm.levels.reading, Reading::Read(2));
    assert_eq!(
        farm.production_methods.reading,
        Reading::Missing { reason: None }
    );
    assert_eq!(farm.staffing.reading, Reading::Missing { reason: None });
    assert_eq!(farm.employees.reading, Reading::Missing { reason: None });
}
