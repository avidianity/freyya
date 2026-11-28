use serde::Deserialize;

use crate::{Observation, Unit};

#[derive(Debug, Clone, PartialEq)]
pub struct Construction {
    pub government_queue: Vec<QueuedConstruction>,
    pub private_queue: Vec<QueuedConstruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QueuedConstruction {
    pub building: String,
    pub state: u32,
    pub construction_left: Observation<f64>,
    pub construction_speed: Observation<f64>,
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

pub(crate) fn construction(
    country: u32,
    government_queue: Option<&SavedQueue>,
    private_queue: Option<&SavedQueue>,
) -> Construction {
    Construction {
        government_queue: queue(country, "government_queue", government_queue),
        private_queue: queue(country, "private_queue", private_queue),
    }
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
                construction_left: Observation::new(
                    element.construction_left,
                    Unit::ConstructionPoints,
                    source("construction_left"),
                ),
                construction_speed: Observation::new(
                    element.construction_speed,
                    Unit::ConstructionPointsPerWeek,
                    source("construction_speed"),
                ),
            }
        })
        .collect()
}
