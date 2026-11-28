use serde::{Deserialize, Serialize};

use crate::observation::{Observation, Unit, observe};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Construction {
    pub capacity: Observation,
    pub government_queue: Vec<QueuedConstruction>,
    pub private_queue: Vec<QueuedConstruction>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QueuedConstruction {
    pub building: String,
    pub state: u32,
    pub construction_left: Observation,
    pub construction_speed: Observation,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SavedQueue {
    #[serde(default)]
    construction_elements: Vec<SavedElement>,
}

#[derive(Debug, Deserialize)]
struct SavedElement {
    #[serde(rename = "type")]
    building: String,
    state: u32,
    construction_left: Option<f64>,
    construction_speed: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SavedAi {
    country: u32,
    spending_variables: Option<SavedSpending>,
}

#[derive(Debug, Deserialize)]
struct SavedSpending {
    construction_output: Option<f64>,
}

pub(crate) fn construction<'a>(
    country: u32,
    government_queue: Option<&SavedQueue>,
    private_queue: Option<&SavedQueue>,
    ai: impl IntoIterator<Item = (u32, &'a SavedAi)>,
) -> Construction {
    Construction {
        capacity: capacity(country, ai),
        government_queue: queue(country, "government_queue", government_queue),
        private_queue: queue(country, "private_queue", private_queue),
    }
}

fn capacity<'a>(country: u32, ai: impl IntoIterator<Item = (u32, &'a SavedAi)>) -> Observation {
    let player_ai = ai.into_iter().find(|(_, entry)| entry.country == country);
    let (source, value) = match player_ai {
        Some((id, entry)) => (
            format!("ai.database.{id}.spending_variables.construction_output"),
            entry
                .spending_variables
                .as_ref()
                .and_then(|spending| spending.construction_output),
        ),
        None => (
            format!(
                "ai.database.<entry for country {country}>.spending_variables.construction_output"
            ),
            None,
        ),
    };
    observe(value, Unit::ConstructionPointsPerWeek, source)
}

fn queue(country: u32, name: &str, saved: Option<&SavedQueue>) -> Vec<QueuedConstruction> {
    let Some(saved) = saved else {
        return Vec::new();
    };
    saved
        .construction_elements
        .iter()
        .enumerate()
        .map(|(index, element)| {
            let source = |field: &str| {
                format!(
                    "country_manager.database.{country}.{name}.construction_elements[{index}].{field}"
                )
            };
            QueuedConstruction {
                building: element.building.clone(),
                state: element.state,
                construction_left: observe(
                    element.construction_left,
                    Unit::ConstructionPoints,
                    source("construction_left"),
                ),
                construction_speed: observe(
                    element.construction_speed,
                    Unit::ConstructionPointsPerWeek,
                    source("construction_speed"),
                ),
            }
        })
        .collect()
}
