use std::collections::{BTreeMap, BTreeSet, HashMap};

use serde::{Deserialize, Serialize};

use crate::buildings_jobs::{Database, SaveBuilding, SaveState};
use crate::money::Budget;
use crate::observation::{Observation, Unit};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GoodsMarket {
    pub market: u32,
    pub owner: Option<String>,
    pub members: Vec<String>,
    pub goods: Vec<MarketGood>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct MarketGood {
    pub good: u16,
    pub price: Observation,
    pub sell_orders_from_buildings: Observation,
    pub buy_orders_from_buildings: Observation,
    pub buy_orders_from_pops: Observation,
    pub net_exports: Observation,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct GoodAmounts {
    #[serde(default)]
    goods: HashMap<u16, GoodAmount>,
}

#[derive(Debug, Deserialize)]
struct GoodAmount {
    value: f64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SaveMarket {
    owner: u32,
}

pub(crate) struct MarketCountry<'a> {
    pub(crate) id: u32,
    pub(crate) tag: Option<&'a str>,
    pub(crate) market: Option<u32>,
    pub(crate) budget: Option<&'a Budget>,
}

pub(crate) struct MarketSections<'a> {
    pub(crate) markets: &'a Database<SaveMarket>,
    pub(crate) states: &'a Database<SaveState>,
    pub(crate) buildings: &'a Database<SaveBuilding>,
}

#[derive(Default)]
struct BuildingOrders {
    sell: HashMap<u16, f64>,
    buy: HashMap<u16, f64>,
}

pub(crate) fn player_goods_market(
    player: u32,
    countries: &[MarketCountry<'_>],
    sections: &MarketSections<'_>,
) -> Option<GoodsMarket> {
    let market = countries
        .iter()
        .find(|country| country.id == player)?
        .market?;
    let owner_id = sections
        .markets
        .entries()
        .find(|(id, _)| *id == market)
        .map(|(_, record)| record.owner);
    let owner = owner_id.and_then(|id| countries.iter().find(|country| country.id == id));
    let member_ids: BTreeSet<u32> = countries
        .iter()
        .filter(|country| country.market == Some(market))
        .map(|country| country.id)
        .collect();
    let mut members: Vec<String> = countries
        .iter()
        .filter(|country| member_ids.contains(&country.id))
        .filter_map(|country| country.tag.map(str::to_owned))
        .collect();
    members.sort();
    let market_states: BTreeMap<u32, &SaveState> = sections
        .states
        .entries()
        .filter(|(_, state)| state.country.is_some_and(|id| member_ids.contains(&id)))
        .collect();
    let orders = building_orders(sections.buildings, &market_states);
    let mut net_exports = HashMap::new();
    for state in market_states.values() {
        add_amounts(&mut net_exports, state.trade.as_ref());
    }
    let prices = owner
        .and_then(|owner| owner.budget)
        .and_then(|budget| budget.current_price_report.as_ref());
    let mut goods: BTreeSet<u16> = prices
        .map(|report| report.goods.keys().copied().collect())
        .unwrap_or_default();
    goods.extend(
        orders
            .sell
            .keys()
            .chain(orders.buy.keys())
            .chain(net_exports.keys()),
    );
    let goods = goods
        .into_iter()
        .map(|good| MarketGood {
            good,
            price: price(good, owner_id, prices),
            sell_orders_from_buildings: building_flow(good, market, "output_goods", &orders.sell),
            buy_orders_from_buildings: building_flow(good, market, "input_goods", &orders.buy),
            buy_orders_from_pops: Observation::missing(
                "the Save stores pop spending as money, not as goods",
                Unit::UnitsPerWeek,
                "pops.database.*.weekly_budget".to_owned(),
            ),
            net_exports: Observation::or_missing(
                net_exports.get(&good).copied(),
                "no state in the market has a trade entry for this good",
                Unit::TradeVolume,
                format!(
                    "states.database.*.trade.goods.{good}.value, summed over the states of market {market}"
                ),
            ),
        })
        .collect();
    Some(GoodsMarket {
        market,
        owner: owner.and_then(|owner| owner.tag.map(str::to_owned)),
        members,
        goods,
    })
}

fn building_orders(
    buildings: &Database<SaveBuilding>,
    market_states: &BTreeMap<u32, &SaveState>,
) -> BuildingOrders {
    let mut orders = BuildingOrders::default();
    let in_market = buildings
        .entries()
        .map(|(_, building)| building)
        .filter(|building| market_states.contains_key(&building.state));
    for building in in_market {
        add_amounts(&mut orders.sell, building.output_goods.as_ref());
        add_amounts(&mut orders.buy, building.input_goods.as_ref());
    }
    orders
}

fn add_amounts(totals: &mut HashMap<u16, f64>, amounts: Option<&GoodAmounts>) {
    for (good, amount) in amounts.into_iter().flat_map(|amounts| &amounts.goods) {
        *totals.entry(*good).or_default() += amount.value;
    }
}

fn building_flow(good: u16, market: u32, field: &str, totals: &HashMap<u16, f64>) -> Observation {
    Observation::or_missing(
        totals.get(&good).copied(),
        format!("no building in the market has this good in its {field}"),
        Unit::UnitsPerWeek,
        format!(
            "building_manager.database.*.{field}.goods.{good}.value, summed over buildings in the states of market {market}"
        ),
    )
}

fn price(good: u16, owner: Option<u32>, prices: Option<&GoodAmounts>) -> Observation {
    let Some(owner) = owner else {
        return Observation::missing(
            "the Save names no owner for the player's market",
            Unit::PoundsPerUnit,
            "market_manager.database".to_owned(),
        );
    };
    Observation::or_missing(
        prices
            .and_then(|report| report.goods.get(&good))
            .map(|amount| amount.value),
        "the market owner's price report has no entry for this good",
        Unit::PoundsPerUnit,
        format!("country_manager.database.{owner}.budget.current_price_report.goods.{good}.value"),
    )
}
