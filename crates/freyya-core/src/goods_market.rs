use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::marker::PhantomData;

use serde::de::value::MapAccessDeserializer;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Debug, Clone, PartialEq)]
pub struct GoodsMarket {
    pub market: u32,
    pub owner: Option<String>,
    pub members: Vec<String>,
    pub goods: Vec<MarketGood>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarketGood {
    pub good: u16,
    pub price: Observation,
    pub sell_orders_from_buildings: Observation,
    pub buy_orders_from_buildings: Observation,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Observation {
    pub value: Value,
    pub unit: Unit,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Read(f64),
    Missing { reason: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    PoundsPerUnit,
    UnitsPerWeek,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::PoundsPerUnit => f.write_str("£ per unit"),
            Unit::UnitsPerWeek => f.write_str("units per week"),
        }
    }
}

impl fmt::Display for Observation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Value::Read(value) => write!(f, "{value} {} [{}]", self.unit, self.source),
            Value::Missing { reason } => {
                write!(f, "missing ({reason}) [{}]", self.source)
            }
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct CountryMarket {
    market: Option<u32>,
    budget: Option<BudgetPrices>,
}

#[derive(Debug, Deserialize)]
struct BudgetPrices {
    current_price_report: Option<GoodAmounts>,
}

impl CountryMarket {
    pub(crate) fn read_field<'de, A: MapAccess<'de>>(
        &mut self,
        key: &str,
        map: &mut A,
    ) -> Result<bool, A::Error> {
        match key {
            "market" => self.market = Some(map.next_value()?),
            "budget" => self.budget = Some(map.next_value()?),
            _ => return Ok(false),
        }
        Ok(true)
    }
}

pub(crate) struct MarketCountry<'a> {
    pub(crate) id: u32,
    pub(crate) tag: Option<&'a str>,
    pub(crate) market: &'a CountryMarket,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Markets {
    database: HashMap<u32, Slot<MarketRecord>>,
}

#[derive(Debug, Deserialize)]
struct MarketRecord {
    owner: u32,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct States {
    database: HashMap<u32, Slot<StateRecord>>,
}

#[derive(Debug, Deserialize)]
struct StateRecord {
    country: Option<u32>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Buildings {
    database: HashMap<u32, Slot<BuildingRecord>>,
}

#[derive(Debug, Deserialize)]
struct BuildingRecord {
    state: Option<u32>,
    input_goods: Option<GoodAmounts>,
    output_goods: Option<GoodAmounts>,
}

#[derive(Debug, Default, Deserialize)]
struct GoodAmounts {
    #[serde(default)]
    goods: HashMap<u16, GoodAmount>,
}

#[derive(Debug, Deserialize)]
struct GoodAmount {
    value: f64,
}

#[derive(Debug)]
struct Slot<T>(Option<T>);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Slot<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(SlotVisitor(PhantomData))
    }
}

struct SlotVisitor<T>(PhantomData<T>);

impl<'de, T: Deserialize<'de>> Visitor<'de> for SlotVisitor<T> {
    type Value = Slot<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an object or none")
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        match value {
            "none" => Ok(Slot(None)),
            other => Err(E::invalid_value(de::Unexpected::Str(other), &self)),
        }
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        T::deserialize(MapAccessDeserializer::new(map)).map(|value| Slot(Some(value)))
    }
}

pub(crate) struct MarketSections<'a> {
    pub(crate) markets: &'a Markets,
    pub(crate) states: &'a States,
    pub(crate) buildings: &'a Buildings,
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
        .market
        .market?;
    let owner_id = sections
        .markets
        .database
        .get(&market)
        .and_then(|record| record.0.as_ref())
        .map(|record| record.owner);
    let owner = owner_id.and_then(|id| countries.iter().find(|country| country.id == id));
    let member_ids: BTreeSet<u32> = countries
        .iter()
        .filter(|country| country.market.market == Some(market))
        .map(|country| country.id)
        .collect();
    let mut members: Vec<String> = countries
        .iter()
        .filter(|country| member_ids.contains(&country.id))
        .filter_map(|country| country.tag.map(str::to_owned))
        .collect();
    members.sort();
    let market_states: BTreeSet<u32> = sections
        .states
        .database
        .iter()
        .filter_map(|(id, state)| {
            let country = state.0.as_ref()?.country?;
            member_ids.contains(&country).then_some(*id)
        })
        .collect();
    let orders = building_orders(sections.buildings, &market_states);
    let prices = owner
        .and_then(|owner| owner.market.budget.as_ref())
        .and_then(|budget| budget.current_price_report.as_ref());
    let mut goods: BTreeSet<u16> = prices
        .map(|report| report.goods.keys().copied().collect())
        .unwrap_or_default();
    goods.extend(orders.sell.keys().chain(orders.buy.keys()));
    let goods = goods
        .into_iter()
        .map(|good| MarketGood {
            good,
            price: price(good, owner_id, prices),
            sell_orders_from_buildings: building_flow(good, market, "output_goods", &orders.sell),
            buy_orders_from_buildings: building_flow(good, market, "input_goods", &orders.buy),
        })
        .collect();
    Some(GoodsMarket {
        market,
        owner: owner.and_then(|owner| owner.tag.map(str::to_owned)),
        members,
        goods,
    })
}

fn building_orders(buildings: &Buildings, market_states: &BTreeSet<u32>) -> BuildingOrders {
    let mut orders = BuildingOrders::default();
    let in_market = buildings
        .database
        .values()
        .filter_map(|building| building.0.as_ref())
        .filter(|building| {
            building
                .state
                .is_some_and(|state| market_states.contains(&state))
        });
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
    Observation {
        value: Value::Read(totals.get(&good).copied().unwrap_or_default()),
        unit: Unit::UnitsPerWeek,
        source: format!(
            "building_manager.database.*.{field}.goods.{good}.value, summed over buildings in the states of market {market}"
        ),
    }
}

fn price(good: u16, owner: Option<u32>, prices: Option<&GoodAmounts>) -> Observation {
    let Some(owner) = owner else {
        return Observation {
            value: Value::Missing {
                reason: "the Save names no owner for the player's market".to_owned(),
            },
            unit: Unit::PoundsPerUnit,
            source: "market_manager.database".to_owned(),
        };
    };
    let source =
        format!("country_manager.database.{owner}.budget.current_price_report.goods.{good}.value");
    let value = match prices.and_then(|report| report.goods.get(&good)) {
        Some(amount) => Value::Read(amount.value),
        None => Value::Missing {
            reason: "the market owner's price report has no entry for this good".to_owned(),
        },
    };
    Observation {
        value,
        unit: Unit::PoundsPerUnit,
        source,
    }
}
