use serde::Serialize;

use crate::observation::{Observation, Unit};
use crate::save::SaveQueue;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Construction {
    pub construction_in_use: Observation,
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

pub(crate) fn construction(
    country: u32,
    government_queue: Option<&SaveQueue>,
    private_queue: Option<&SaveQueue>,
) -> Construction {
    Construction {
        construction_in_use: construction_in_use(country, [government_queue, private_queue]),
        government_queue: queue(country, "government_queue", government_queue),
        private_queue: queue(country, "private_queue", private_queue),
    }
}

fn construction_in_use(country: u32, queues: [Option<&SaveQueue>; 2]) -> Observation {
    let speeds: Vec<f64> = queues
        .into_iter()
        .flatten()
        .flat_map(|queue| &queue.construction_elements)
        .filter_map(|element| element.base_construction_speed)
        .collect();
    let total = (!speeds.is_empty()).then(|| speeds.iter().sum());
    Observation::of(
        total,
        Unit::ConstructionPointsPerWeek,
        format!(
            "country_manager.database.{country}.government_queue and private_queue construction_elements[*].base_construction_speed (sum)"
        ),
    )
}

fn queue(country: u32, name: &str, saved: Option<&SaveQueue>) -> Vec<QueuedConstruction> {
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
                construction_left: Observation::of(
                    element.construction_left,
                    Unit::ConstructionPoints,
                    source("construction_left"),
                ),
                construction_speed: Observation::of(
                    element.base_construction_speed,
                    Unit::ConstructionPointsPerWeek,
                    source("base_construction_speed"),
                ),
            }
        })
        .collect()
}
