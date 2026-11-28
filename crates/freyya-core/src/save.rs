use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use jomini::envelope::JominiTextFile;
use serde::de::value::MapAccessDeserializer;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::Refusal;

#[derive(Debug, Deserialize)]
pub(crate) struct SaveGamestate {
    pub(crate) meta_data: SaveMetaData,
    pub(crate) player_manager: Database<SavePlayer>,
    pub(crate) country_manager: Database<SaveCountry>,
    #[serde(default)]
    pub(crate) states: Database<SaveState>,
    #[serde(default)]
    pub(crate) building_manager: Database<SaveBuilding>,
    #[serde(default)]
    pub(crate) pops: Database<SavePop>,
    #[serde(default)]
    pub(crate) market_manager: Database<SaveMarket>,
}

impl SaveGamestate {
    pub(crate) fn parse(melted: &[u8]) -> Result<Self, Refusal> {
        let mut file = JominiTextFile::from_reader(melted).map_err(|error| Refusal::Damaged {
            detail: format!("the decoded Save has no text header: {error}"),
        })?;
        file.deserializer()
            .deserialize()
            .map_err(|error| Refusal::Damaged {
                detail: format!("the decoded Save does not have the expected shape: {error}"),
            })
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveMetaData {
    pub(crate) version: String,
    pub(crate) game_date: String,
    pub(crate) name: String,
    #[serde(default)]
    pub(crate) mods: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SavePlayer {
    pub(crate) country: u32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveCountry {
    #[serde(default)]
    pub(crate) definition: Option<String>,
    #[serde(default)]
    pub(crate) budget: Option<SaveBudget>,
    #[serde(default)]
    pub(crate) pop_statistics: Option<SavePopStatistics>,
    #[serde(default)]
    pub(crate) government_queue: Option<SaveQueue>,
    #[serde(default)]
    pub(crate) private_queue: Option<SaveQueue>,
    pub(crate) market: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SaveBudget {
    #[serde(default)]
    pub(crate) money: Option<f64>,
    #[serde(default)]
    pub(crate) weekly_income: Vec<f64>,
    #[serde(default)]
    pub(crate) weekly_expenses: Vec<f64>,
    #[serde(default)]
    pub(crate) principal: Option<f64>,
    #[serde(default)]
    pub(crate) current_price_report: Option<SaveGoodAmounts>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SaveGoodAmounts {
    #[serde(default)]
    pub(crate) goods: HashMap<u16, SaveGoodAmount>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveGoodAmount {
    pub(crate) value: f64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveState {
    pub(crate) country: Option<u32>,
    pub(crate) region: String,
    #[serde(default)]
    pub(crate) trade: Option<SaveGoodAmounts>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveBuilding {
    pub(crate) building: String,
    pub(crate) state: u32,
    pub(crate) levels: Option<u32>,
    pub(crate) production_methods: Option<Vec<String>>,
    pub(crate) staffing: Option<f64>,
    #[serde(default)]
    pub(crate) input_goods: Option<SaveGoodAmounts>,
    #[serde(default)]
    pub(crate) output_goods: Option<SaveGoodAmounts>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SavePop {
    pub(crate) workforce: Option<u64>,
    pub(crate) workplace: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SavePopStatistics {
    pub(crate) population_salaried_workforce: Option<f64>,
    pub(crate) population_subsisting_workforce: Option<f64>,
    pub(crate) population_unemployed_workforce: Option<f64>,
    pub(crate) vacancies_new_culture: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveMarket {
    pub(crate) owner: u32,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SaveQueue {
    #[serde(default)]
    pub(crate) construction_elements: Vec<SaveQueueElement>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveQueueElement {
    #[serde(rename = "type")]
    pub(crate) building: String,
    pub(crate) state: u32,
    pub(crate) construction_left: Option<f64>,
    pub(crate) base_construction_speed: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Database<T> {
    database: HashMap<u32, MaybeNone<T>>,
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

    pub(crate) fn get(&self, id: u32) -> Option<&T> {
        self.database.get(&id).and_then(|entry| entry.0.as_ref())
    }
}

#[derive(Debug)]
struct MaybeNone<T>(Option<T>);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for MaybeNone<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(MaybeNoneVisitor(PhantomData))
    }
}

struct MaybeNoneVisitor<T>(PhantomData<T>);

impl<'de, T: Deserialize<'de>> Visitor<'de> for MaybeNoneVisitor<T> {
    type Value = MaybeNone<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an entry or none")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        match value {
            "none" => Ok(MaybeNone(None)),
            other => Err(E::invalid_value(de::Unexpected::Str(other), &self)),
        }
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        T::deserialize(MapAccessDeserializer::new(map)).map(|entry| MaybeNone(Some(entry)))
    }
}
