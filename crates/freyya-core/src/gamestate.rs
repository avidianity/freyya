use std::str::FromStr;

use crate::goods_market::{self, MarketCountry, MarketSections};
use crate::save::{SaveCountry, SaveGamestate};
use crate::{GameDate, Money, PlayerCountry, Refusal, Snapshot, buildings_jobs, construction};

const CHECKED_GAME_VERSIONS: &[&str] = &["1.15.2"];

pub(crate) fn into_snapshot(save: SaveGamestate) -> Result<Snapshot, Refusal> {
    if !CHECKED_GAME_VERSIONS.contains(&save.meta_data.version.as_str()) {
        return Err(Refusal::UnsupportedVersion {
            version: save.meta_data.version,
        });
    }
    if !save.meta_data.mods.is_empty() {
        return Err(Refusal::Modded {
            mods: save.meta_data.mods,
        });
    }
    let game_date = parse_game_date(&save.meta_data.game_date)?;
    let (country_id, country, tag) = player_country(&save)?;
    let player_country = PlayerCountry {
        tag: tag.to_owned(),
        name: save.meta_data.name.clone(),
    };
    let money = Money::from_budget(country_id, country.budget.as_ref());
    let buildings_and_jobs = buildings_jobs::buildings_and_jobs(
        country_id,
        &save.states,
        &save.building_manager,
        &save.pops,
        country.pop_statistics.as_ref(),
    );
    let construction = construction::construction(
        country_id,
        country.government_queue.as_ref(),
        country.private_queue.as_ref(),
    );
    let goods_market = goods_market::player_goods_market(
        country_id,
        &market_countries(&save),
        &MarketSections {
            markets: &save.market_manager,
            states: &save.states,
            buildings: &save.building_manager,
        },
    );
    Ok(Snapshot {
        game_version: save.meta_data.version,
        game_date,
        player_country,
        money,
        buildings_and_jobs,
        construction,
        goods_market,
    })
}

fn market_countries(save: &SaveGamestate) -> Vec<MarketCountry<'_>> {
    save.country_manager
        .entries()
        .map(|(id, country)| MarketCountry {
            id,
            tag: country.definition.as_deref(),
            market: country.market,
            budget: country.budget.as_ref(),
        })
        .collect()
}

fn player_country(save: &SaveGamestate) -> Result<(u32, &SaveCountry, &str), Refusal> {
    let mut players = save.player_manager.entries().map(|(_, player)| player);
    let player = match (players.next(), players.next()) {
        (Some(player), None) => player,
        (None, _) => return Err(Refusal::damaged("the Save names no player")),
        (Some(_), Some(_)) => return Err(Refusal::damaged("the Save names more than one player")),
    };
    match save.country_manager.get(player.country) {
        Some(
            country @ SaveCountry {
                definition: Some(tag),
                ..
            },
        ) => Ok((player.country, country, tag)),
        _ => Err(Refusal::damaged(format!(
            "the player's country {} is not in the Save",
            player.country
        ))),
    }
}

fn parse_game_date(text: &str) -> Result<GameDate, Refusal> {
    let mut parts = text.split('.');
    Ok(GameDate {
        year: date_part(text, parts.next())?,
        month: date_part(text, parts.next())?,
        day: date_part(text, parts.next())?,
    })
}

fn date_part<T>(text: &str, part: Option<&str>) -> Result<T, Refusal>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let detail = format!("the game date {text:?} is not a date");
    match part {
        Some(part) => part
            .parse()
            .map_err(|error| Refusal::damaged_by(detail, error)),
        None => Err(Refusal::damaged(detail)),
    }
}
