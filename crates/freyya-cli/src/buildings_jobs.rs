use std::fmt::Display;

use freyya_core::Catalog;
use freyya_core::buildings_jobs::{Building, BuildingsAndJobs, Observation};

use crate::building_label;

pub(crate) fn print_buildings_and_jobs(area: &BuildingsAndJobs, catalog: Option<&Catalog>) {
    println!("Jobs (country-wide):");
    for (label, observation) in [
        ("Salaried workforce", &area.jobs.salaried_workforce),
        ("Subsisting workforce", &area.jobs.subsisting_workforce),
        ("Unemployed workforce", &area.jobs.unemployed_workforce),
        ("Vacancies", &area.jobs.vacancies),
    ] {
        println!(
            "  {label:<21} {:>16}  [{}]",
            shown(observation),
            observation.source
        );
    }
    for state in &area.states {
        println!(
            "Buildings in {} (state {}): [building_manager.database.<id>; employees from pops.database.*.workforce by workplace]",
            state.region, state.state_id
        );
        for building in &state.buildings {
            print_building(building, catalog);
        }
    }
}

fn print_building(building: &Building, catalog: Option<&Catalog>) {
    let methods = match &building.production_methods.value {
        Some(methods) => methods.join(" "),
        None => "missing".to_owned(),
    };
    println!(
        "  {:>8} {:<40} levels {:<4} staffing {:<24} employees {:<14} methods {methods}",
        building.id,
        building_label(&building.kind, catalog),
        value_or_missing(&building.levels),
        shown(&building.staffing),
        shown(&building.employees),
    );
}

fn shown<T: Display>(observation: &Observation<T>) -> String {
    match &observation.value {
        Some(value) => format!("{value} {}", observation.unit),
        None => "missing".to_owned(),
    }
}

fn value_or_missing<T: Display>(observation: &Observation<T>) -> String {
    match &observation.value {
        Some(value) => value.to_string(),
        None => "missing".to_owned(),
    }
}
