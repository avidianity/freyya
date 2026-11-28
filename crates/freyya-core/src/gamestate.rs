use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

use jomini::envelope::JominiTextFile;
use serde::de::value::MapAccessDeserializer;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::buildings_jobs::{self, Database, SaveBuilding, SavePop, SavePopStatistics, SaveState};
use crate::construction::{self, SavedQueue};
use crate::goods_market::{self, MarketCountry, MarketSections, SaveMarket};
use crate::money::Budget;
use crate::{GameDate, Money, PlayerCountry, Refusal, Snapshot};

const CHECKED_GAME_VERSIONS: &[&str] = &["1.15.2"];

#[derive(Debug, Deserialize)]
pub(crate) struct Gamestate {
    meta_data: MetaData,
    player_manager: Manager<Player>,
    country_manager: Manager<MaybeNone<Country>>,
    #[serde(default)]
    states: Database<SaveState>,
    #[serde(default)]
    building_manager: Database<SaveBuilding>,
    #[serde(default)]
    pops: Database<SavePop>,
    #[serde(default)]
    market_manager: Database<SaveMarket>,
}

#[derive(Debug, Deserialize)]
struct MetaData {
    version: String,
    game_date: String,
    name: String,
    #[serde(default)]
    mods: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Manager<T> {
    database: HashMap<u32, T>,
}

#[derive(Debug, Deserialize)]
struct Player {
    country: u32,
}

#[derive(Debug)]
struct MaybeNone<T>(Option<T>);

#[derive(Debug, Deserialize)]
struct Country {
    #[serde(default)]
    definition: Option<String>,
    #[serde(default)]
    budget: Option<Budget>,
    #[serde(default)]
    pop_statistics: Option<SavePopStatistics>,
    #[serde(default)]
    government_queue: Option<SavedQueue>,
    #[serde(default)]
    private_queue: Option<SavedQueue>,
    market: Option<u32>,
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for MaybeNone<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(MaybeNoneVisitor(PhantomData))
    }
}

struct MaybeNoneVisitor<T>(PhantomData<T>);

impl<'de, T: Deserialize<'de>> Visitor<'de> for MaybeNoneVisitor<T> {
    type Value = MaybeNone<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an object or none")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        match value {
            "none" => Ok(MaybeNone(None)),
            other => Err(E::invalid_value(de::Unexpected::Str(other), &self)),
        }
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        T::deserialize(MapAccessDeserializer::new(map)).map(|value| MaybeNone(Some(value)))
    }
}

impl Gamestate {
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

    pub(crate) fn into_snapshot(self) -> Result<Snapshot, Refusal> {
        if !CHECKED_GAME_VERSIONS.contains(&self.meta_data.version.as_str()) {
            return Err(Refusal::UnsupportedVersion {
                version: self.meta_data.version,
            });
        }
        if !self.meta_data.mods.is_empty() {
            return Err(Refusal::Modded {
                mods: self.meta_data.mods,
            });
        }
        let game_date = parse_game_date(&self.meta_data.game_date)?;
        let (country_id, country, tag) = self.player_country()?;
        let player_country = PlayerCountry {
            tag: tag.to_owned(),
            name: self.meta_data.name.clone(),
        };
        let money = Money::from_budget(country_id, country.budget.as_ref());
        let buildings_and_jobs = buildings_jobs::buildings_and_jobs(
            country_id,
            &self.states,
            &self.building_manager,
            &self.pops,
            country.pop_statistics.as_ref(),
        );
        let construction = construction::construction(
            country_id,
            country.government_queue.as_ref(),
            country.private_queue.as_ref(),
        );
        let goods_market = goods_market::player_goods_market(
            country_id,
            &self.market_countries(),
            &MarketSections {
                markets: &self.market_manager,
                states: &self.states,
                buildings: &self.building_manager,
            },
        );
        Ok(Snapshot {
            game_version: self.meta_data.version,
            game_date,
            player_country,
            money,
            buildings_and_jobs,
            construction,
            goods_market,
        })
    }

    fn market_countries(&self) -> Vec<MarketCountry<'_>> {
        self.country_manager
            .database
            .iter()
            .filter_map(|(id, country)| {
                country.0.as_ref().map(|country| MarketCountry {
                    id: *id,
                    tag: country.definition.as_deref(),
                    market: country.market,
                    budget: country.budget.as_ref(),
                })
            })
            .collect()
    }

    fn player_country(&self) -> Result<(u32, &Country, &str), Refusal> {
        let mut players = self.player_manager.database.values();
        let player = match (players.next(), players.next()) {
            (Some(player), None) => player,
            (None, _) => {
                return Err(Refusal::Damaged {
                    detail: "the Save names no player".to_owned(),
                });
            }
            (Some(_), Some(_)) => {
                return Err(Refusal::Damaged {
                    detail: "the Save names more than one player".to_owned(),
                });
            }
        };
        match self.country_manager.database.get(&player.country) {
            Some(MaybeNone(Some(
                country @ Country {
                    definition: Some(tag),
                    ..
                },
            ))) => Ok((player.country, country, tag)),
            _ => Err(Refusal::Damaged {
                detail: format!("the player's country {} is not in the Save", player.country),
            }),
        }
    }
}

fn parse_game_date(text: &str) -> Result<GameDate, Refusal> {
    let invalid = || Refusal::Damaged {
        detail: format!("the game date {text:?} is not a date"),
    };
    let mut parts = text.split('.');
    let mut next = || parts.next().ok_or_else(invalid);
    let year = next()?.parse().map_err(|_| invalid())?;
    let month = next()?.parse().map_err(|_| invalid())?;
    let day = next()?.parse().map_err(|_| invalid())?;
    Ok(GameDate { year, month, day })
}
