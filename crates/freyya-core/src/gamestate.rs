use std::collections::HashMap;
use std::fmt;

use jomini::envelope::JominiTextFile;
use serde::de::{self, IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use crate::{GameDate, PlayerCountry, Refusal, Snapshot};

const CHECKED_GAME_VERSIONS: &[&str] = &["1.15.2"];

#[derive(Debug, Deserialize)]
pub(crate) struct Gamestate {
    meta_data: MetaData,
    player_manager: Manager<Player>,
    country_manager: Manager<MaybeCountry>,
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
struct MaybeCountry(Option<String>);

impl<'de> Deserialize<'de> for MaybeCountry {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(MaybeCountryVisitor)
    }
}

struct MaybeCountryVisitor;

impl<'de> Visitor<'de> for MaybeCountryVisitor {
    type Value = MaybeCountry;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a country or none")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        match value {
            "none" => Ok(MaybeCountry(None)),
            other => Err(E::invalid_value(de::Unexpected::Str(other), &self)),
        }
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut definition = None;
        while let Some(key) = map.next_key::<String>()? {
            if key == "definition" {
                definition = Some(map.next_value::<String>()?);
            } else {
                map.next_value::<IgnoredAny>()?;
            }
        }
        Ok(MaybeCountry(definition))
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
        let tag = self.player_tag()?;
        Ok(Snapshot {
            game_version: self.meta_data.version,
            game_date,
            player_country: PlayerCountry {
                tag,
                name: self.meta_data.name,
            },
        })
    }

    fn player_tag(&self) -> Result<String, Refusal> {
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
            Some(MaybeCountry(Some(tag))) => Ok(tag.clone()),
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
