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
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Unit::PoundsPerUnit => f.write_str("£ per unit"),
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

pub(crate) fn player_goods_market(
    player: u32,
    countries: &[MarketCountry<'_>],
    markets: &Markets,
) -> Option<GoodsMarket> {
    let market = countries
        .iter()
        .find(|country| country.id == player)?
        .market
        .market?;
    let owner_id = markets
        .database
        .get(&market)
        .and_then(|record| record.0.as_ref())
        .map(|record| record.owner);
    let owner = owner_id.and_then(|id| countries.iter().find(|country| country.id == id));
    let mut members: Vec<String> = countries
        .iter()
        .filter(|country| country.market.market == Some(market))
        .filter_map(|country| country.tag.map(str::to_owned))
        .collect();
    members.sort();
    let prices = owner
        .and_then(|owner| owner.market.budget.as_ref())
        .and_then(|budget| budget.current_price_report.as_ref());
    let goods: BTreeSet<u16> = prices
        .map(|report| report.goods.keys().copied().collect())
        .unwrap_or_default();
    let goods = goods
        .into_iter()
        .map(|good| MarketGood {
            good,
            price: price(good, owner_id, prices),
        })
        .collect();
    Some(GoodsMarket {
        market,
        owner: owner.and_then(|owner| owner.tag.map(str::to_owned)),
        members,
        goods,
    })
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
