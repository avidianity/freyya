use std::collections::HashMap;

use serde::Serialize;

use crate::observation::{Observation, Unit};
use crate::save::{Database, SaveBuilding, SavePop, SavePopStatistics, SaveState};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BuildingsAndJobs {
    pub states: Vec<StateBuildings>,
    pub jobs: Jobs,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Jobs {
    pub salaried_workforce: Observation<u64>,
    pub subsisting_workforce: Observation<u64>,
    pub unemployed_workforce: Observation<u64>,
    pub vacancies: Observation<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StateBuildings {
    pub state_id: u32,
    pub region: String,
    pub buildings: Vec<Building>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Building {
    pub id: u32,
    pub kind: String,
    pub levels: Observation<u32>,
    pub production_methods: Observation<Vec<String>>,
    pub staffing: Observation<f64>,
    pub employees: Observation<u64>,
}

#[derive(Clone, Copy)]
enum StoredAs {
    People,
    HundredThousandPeople,
}

pub(crate) fn buildings_and_jobs(
    player_country: u32,
    states: &Database<SaveState>,
    buildings: &Database<SaveBuilding>,
    pops: &Database<SavePop>,
    pop_statistics: Option<&SavePopStatistics>,
) -> BuildingsAndJobs {
    let employees = employees_by_workplace(pops);
    let mut player_states: Vec<StateBuildings> = states
        .entries()
        .filter(|(_, state)| state.country == Some(player_country))
        .map(|(state_id, state)| StateBuildings {
            state_id,
            region: state.region.clone(),
            buildings: Vec::new(),
        })
        .collect();
    player_states.sort_by_key(|state| state.state_id);
    for (id, building) in buildings.entries() {
        if let Some(state) = player_states
            .iter_mut()
            .find(|state| state.state_id == building.state)
        {
            state
                .buildings
                .push(observe_building(id, building, employees.get(&id).copied()));
        }
    }
    for state in &mut player_states {
        state.buildings.sort_by_key(|building| building.id);
    }
    BuildingsAndJobs {
        states: player_states,
        jobs: observe_jobs(player_country, pop_statistics),
    }
}

fn observe_jobs(country: u32, statistics: Option<&SavePopStatistics>) -> Jobs {
    let people = |field: &str, read: fn(&SavePopStatistics) -> Option<f64>, saved_as: StoredAs| {
        let source = format!("country_manager.database.{country}.pop_statistics.{field}");
        Observation::of(
            statistics
                .and_then(read)
                .map(|value| match saved_as {
                    StoredAs::People => value,
                    StoredAs::HundredThousandPeople => value * 100_000.0,
                })
                .map(|people| people.round() as u64),
            Unit::People,
            match saved_as {
                StoredAs::People => source,
                StoredAs::HundredThousandPeople => format!("{source} x 100000"),
            },
        )
    };
    Jobs {
        salaried_workforce: people(
            "population_salaried_workforce",
            |statistics| statistics.population_salaried_workforce,
            StoredAs::People,
        ),
        subsisting_workforce: people(
            "population_subsisting_workforce",
            |statistics| statistics.population_subsisting_workforce,
            StoredAs::HundredThousandPeople,
        ),
        unemployed_workforce: people(
            "population_unemployed_workforce",
            |statistics| statistics.population_unemployed_workforce,
            StoredAs::People,
        ),
        vacancies: people(
            "vacancies_new_culture",
            |statistics| statistics.vacancies_new_culture,
            StoredAs::HundredThousandPeople,
        ),
    }
}

fn employees_by_workplace(pops: &Database<SavePop>) -> HashMap<u32, u64> {
    let mut employees = HashMap::new();
    for (_, pop) in pops.entries() {
        if let Some(workplace) = pop.workplace {
            *employees.entry(workplace).or_insert(0) += pop.workforce.unwrap_or(0);
        }
    }
    employees
}

fn observe_building(id: u32, building: &SaveBuilding, employees: Option<u64>) -> Building {
    let source = |field: &str| format!("building_manager.database.{id}.{field}");
    Building {
        id,
        kind: building.building.clone(),
        levels: Observation::of(building.levels, Unit::Levels, source("levels")),
        production_methods: Observation::of(
            building.production_methods.clone(),
            Unit::ProductionMethods,
            source("production_methods"),
        ),
        staffing: Observation::of(building.staffing, Unit::StaffedLevels, source("staffing")),
        employees: Observation::of(
            employees,
            Unit::People,
            format!("pops.database.*.workforce where workplace={id}"),
        ),
    }
}
