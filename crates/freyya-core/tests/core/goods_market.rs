use freyya_core::goods_market::{GoodsMarket, MarketGood};
use freyya_core::{Observation, Reading, Unit};

use crate::helpers::{player_save, read_manual_save, read_text_save};

const TANKS: u16 = 3;
const GRAIN: u16 = 7;
const CLOTHES: u16 = 12;
const IRON: u16 = 25;
const TOOLS: u16 = 35;

fn manual_save_market() -> GoodsMarket {
    read_manual_save()
        .goods_market
        .expect("the manual Save names the player's market")
}

fn good(market: &GoodsMarket, good: u16) -> &MarketGood {
    market
        .goods
        .iter()
        .find(|entry| entry.good == good)
        .unwrap_or_else(|| panic!("good {good} is in the market"))
}

fn read_value(observation: &Observation) -> f64 {
    match &observation.reading {
        Reading::Read(value) => *value,
        Reading::Missing { reason } => panic!("expected a read value, got missing: {reason:?}"),
    }
}

fn assert_missing(observation: &Observation) {
    assert!(
        matches!(observation.reading, Reading::Missing { .. }),
        "expected missing, got {observation:?}"
    );
}

fn assert_close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-4,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn manual_save_gives_the_price_of_each_good_in_the_players_market() {
    let market = manual_save_market();
    let grain = good(&market, GRAIN);
    assert_close(read_value(&grain.price), 11.6228);
    assert_eq!(grain.price.unit, Unit::PoundsPerUnit);
    assert_eq!(
        grain.price.source,
        "country_manager.database.24.budget.current_price_report.goods.7.value"
    );
    assert_close(read_value(&good(&market, CLOTHES).price), 37.67137);
    assert_close(read_value(&good(&market, IRON).price), 40.0852);
    assert_eq!(market.goods.len(), 54);
}

#[test]
fn manual_save_gives_sell_and_buy_orders_of_buildings_in_the_players_market() {
    let market = manual_save_market();
    let iron = good(&market, IRON);
    assert_close(read_value(&iron.sell_orders_from_buildings), 314.6);
    assert_close(read_value(&iron.buy_orders_from_buildings), 143.01275);
    assert_eq!(iron.sell_orders_from_buildings.unit, Unit::UnitsPerWeek);
    assert_eq!(
        iron.sell_orders_from_buildings.source,
        "building_manager.database.*.output_goods.goods.25.value, summed over buildings in the states of market 8"
    );
    assert_eq!(
        iron.buy_orders_from_buildings.source,
        "building_manager.database.*.input_goods.goods.25.value, summed over buildings in the states of market 8"
    );
    let grain = good(&market, GRAIN);
    assert_close(read_value(&grain.sell_orders_from_buildings), 221.87076);
    assert_close(read_value(&grain.buy_orders_from_buildings), 14.47342);
    let clothes = good(&market, CLOTHES);
    assert_close(read_value(&clothes.sell_orders_from_buildings), 173.50944);
    assert_missing(&clothes.buy_orders_from_buildings);
    let tanks = good(&market, TANKS);
    assert_missing(&tanks.sell_orders_from_buildings);
    assert_missing(&tanks.buy_orders_from_buildings);
}

#[test]
fn manual_save_gives_net_exports_of_the_players_market() {
    let market = manual_save_market();
    let iron = good(&market, IRON);
    assert_close(read_value(&iron.net_exports), 30.0);
    assert_eq!(iron.net_exports.unit, Unit::TradeVolume);
    assert_eq!(
        iron.net_exports.source,
        "states.database.*.trade.goods.25.value, summed over the states of market 8"
    );
    assert_close(read_value(&good(&market, TOOLS).net_exports), -12.0);
    assert_missing(&good(&market, TANKS).net_exports);
}

fn hand_made_market_save(country: &str) -> Vec<u8> {
    player_save(
        country,
        "states={ database={ 0=none 1={ country=7 region=\"STATE_TEST\" } } }\n\
         market_manager={ database={ 3={ owner=7 } } }\n\
         building_manager={ database={ 0=none 5={ building=\"building_textile_mill\" state=1 \
         output_goods={ behavior=keep_zeros goods={ 12={ value=5.5 } } } } } }",
    )
}

fn hand_made_market(country: &str) -> GoodsMarket {
    read_text_save(&hand_made_market_save(country))
        .goods_market
        .expect("the hand-made Save names the player's market")
}

#[test]
fn price_is_missing_not_zero_when_the_save_has_no_price_report() {
    let market = hand_made_market("market=3");
    assert_eq!(market.owner.as_deref(), Some("TST"));
    let clothes = good(&market, CLOTHES);
    assert_close(read_value(&clothes.sell_orders_from_buildings), 5.5);
    assert!(
        matches!(clothes.price.reading, Reading::Missing { .. }),
        "got {:?}",
        clothes.price
    );
}

#[test]
fn building_orders_and_net_exports_are_missing_not_zero_when_the_save_has_no_entry() {
    let market = hand_made_market("market=3");
    let clothes = good(&market, CLOTHES);
    assert!(
        matches!(
            clothes.buy_orders_from_buildings.reading,
            Reading::Missing { .. }
        ),
        "got {:?}",
        clothes.buy_orders_from_buildings
    );
    assert!(
        matches!(clothes.net_exports.reading, Reading::Missing { .. }),
        "got {:?}",
        clothes.net_exports
    );
}

#[test]
fn goods_market_is_missing_when_the_save_names_no_market_for_the_player() {
    let save = hand_made_market_save("");
    let snapshot = read_text_save(&save);
    assert_eq!(snapshot.goods_market, None);
}

#[test]
fn buy_orders_of_pops_are_missing_because_the_save_does_not_store_them() {
    let market = manual_save_market();
    let grain = good(&market, GRAIN);
    assert!(
        matches!(grain.buy_orders_from_pops.reading, Reading::Missing { .. }),
        "got {:?}",
        grain.buy_orders_from_pops
    );
    assert_eq!(grain.buy_orders_from_pops.unit, Unit::UnitsPerWeek);
}
