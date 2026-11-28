use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use serde::de::value::MapAccessDeserializer;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, PartialEq)]
pub struct BuildingsAndJobs {
    pub states: Vec<StateBuildings>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateBuildings {
    pub state_id: u32,
    pub region: String,
    pub buildings: Vec<Building>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Building {
    pub id: u32,
    pub kind: String,
    pub levels: Observation<u32>,
    pub production_methods: Observation<Vec<String>>,
    pub staffing: Observation<f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Observation<T> {
    pub value: Option<T>,
    pub unit: Unit,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    Levels,
    StaffedLevels,
    ProductionMethods,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
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
    fn entries(&self) -> impl Iterator<Item = (u32, &T)> {
        self.database
            .iter()
            .filter_map(|(id, entry)| entry.0.as_ref().map(|entry| (*id, entry)))
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveState {
    country: Option<u32>,
    region: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveBuilding {
    building: String,
    state: u32,
    levels: Option<u32>,
    production_methods: Option<Vec<String>>,
    staffing: Option<f64>,
}

pub(crate) fn buildings_and_jobs(
    player_country: u32,
    states: &Database<SaveState>,
    buildings: &Database<SaveBuilding>,
) -> BuildingsAndJobs {
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
            state.buildings.push(observe_building(id, building));
        }
    }
    for state in &mut player_states {
        state.buildings.sort_by_key(|building| building.id);
    }
    BuildingsAndJobs {
        states: player_states,
    }
}

fn observe_building(id: u32, building: &SaveBuilding) -> Building {
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
    }
}
