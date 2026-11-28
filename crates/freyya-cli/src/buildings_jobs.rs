use freyya_core::Catalog;
use freyya_core::buildings_jobs::{Building, BuildingsAndJobs};

use crate::building_label;

pub(crate) fn print_buildings_and_jobs(area: &BuildingsAndJobs, catalog: Option<&Catalog>) {
    println!("Jobs (country-wide):");
    for (label, observation) in [
        ("Salaried workforce", &area.jobs.salaried_workforce),
        ("Subsisting workforce", &area.jobs.subsisting_workforce),
        ("Unemployed workforce", &area.jobs.unemployed_workforce),
        ("Vacancies", &area.jobs.vacancies),
    ] {
        println!("  {label:<21} {observation}");
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
    println!(
        "  {:>8} {:<40} {:<10} staffing {:<22} employees {:<14} {}",
        building.id,
        building_label(&building.kind, catalog),
        building.levels.shown(),
        building.staffing.shown(),
        building.employees.shown(),
        building.production_methods.shown(),
    );
}
