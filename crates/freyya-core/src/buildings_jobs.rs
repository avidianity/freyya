use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::value::MapAccessDeserializer;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

use crate::goods_market::GoodAmounts;

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

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Observation<T> {
    pub value: Option<T>,
    pub unit: Unit,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Unit {
    People,
    Levels,
    StaffedLevels,
    ProductionMethods,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Unit::People => "people",
            Unit::Levels => "levels",
            Unit::StaffedLevels => "staffed levels",
            Unit::ProductionMethods => "production methods",
        })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Database<T> {
    database: HashMap<u32, MaybeEntry<T>>,
}

#[derive(Debug)]
struct MaybeEntry<T>(Option<T>);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for MaybeEntry<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(MaybeEntryVisitor(PhantomData))
    }
}

struct MaybeEntryVisitor<T>(PhantomData<T>);

impl<'de, T: Deserialize<'de>> Visitor<'de> for MaybeEntryVisitor<T> {
    type Value = MaybeEntry<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an entry or none")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        match value {
            "none" => Ok(MaybeEntry(None)),
            other => Err(E::invalid_value(de::Unexpected::Str(other), &self)),
        }
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        T::deserialize(MapAccessDeserializer::new(map)).map(|entry| MaybeEntry(Some(entry)))
    }
}

impl<T> Default for Database<T> {
    fn default() -> Self {
        Self {
            database: HashMap::new(),
        }
    }
}

impl<T> Database<T> {
    pub(crate) fn entries(&self) -> impl Iterator<Item = (u32, &T)> {
        self.database
            .iter()
            .filter_map(|(id, entry)| entry.0.as_ref().map(|entry| (*id, entry)))
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveState {
    pub(crate) country: Option<u32>,
    region: String,
    #[serde(default)]
    pub(crate) trade: Option<GoodAmounts>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveBuilding {
    building: String,
    pub(crate) state: u32,
    levels: Option<u32>,
    production_methods: Option<Vec<String>>,
    staffing: Option<f64>,
    #[serde(default)]
    pub(crate) input_goods: Option<GoodAmounts>,
    #[serde(default)]
    pub(crate) output_goods: Option<GoodAmounts>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SavePop {
    workforce: Option<u64>,
    workplace: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SavePopStatistics {
    population_salaried_workforce: Option<f64>,
    population_subsisting_workforce: Option<f64>,
    population_unemployed_workforce: Option<f64>,
    vacancies_new_culture: Option<f64>,
}

#[derive(Clone, Copy)]
enum SavedAs {
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
    let people = |field: &str, read: fn(&SavePopStatistics) -> Option<f64>, saved_as: SavedAs| {
        let source = format!("country_manager.database.{country}.pop_statistics.{field}");
        Observation {
            value: statistics
                .and_then(read)
                .map(|value| match saved_as {
                    SavedAs::People => value,
                    SavedAs::HundredThousandPeople => value * 100_000.0,
                })
                .map(|people| people.round() as u64),
            unit: Unit::People,
            source: match saved_as {
                SavedAs::People => source,
                SavedAs::HundredThousandPeople => format!("{source} x 100000"),
            },
        }
    };
    Jobs {
        salaried_workforce: people(
            "population_salaried_workforce",
            |statistics| statistics.population_salaried_workforce,
            SavedAs::People,
        ),
        subsisting_workforce: people(
            "population_subsisting_workforce",
            |statistics| statistics.population_subsisting_workforce,
            SavedAs::HundredThousandPeople,
        ),
        unemployed_workforce: people(
            "population_unemployed_workforce",
            |statistics| statistics.population_unemployed_workforce,
            SavedAs::People,
        ),
        vacancies: people(
            "vacancies_new_culture",
            |statistics| statistics.vacancies_new_culture,
            SavedAs::HundredThousandPeople,
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
        levels: Observation {
            value: building.levels,
            unit: Unit::Levels,
            source: source("levels"),
        },
        production_methods: Observation {
            value: building.production_methods.clone(),
            unit: Unit::ProductionMethods,
            source: source("production_methods"),
        },
        staffing: Observation {
            value: building.staffing,
            unit: Unit::StaffedLevels,
            source: source("staffing"),
        },
        employees: Observation {
            value: employees,
            unit: Unit::People,
            source: format!("pops.database.*.workforce where workplace={id}"),
        },
    }
}
