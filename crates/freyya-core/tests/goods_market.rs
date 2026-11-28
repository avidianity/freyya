use freyya_core::goods_market::{GoodsMarket, MarketGood, Observation, Unit, Value};
use freyya_core::{Rakaly, read_save};

const TANKS: u16 = 3;
const GRAIN: u16 = 7;
const CLOTHES: u16 = 12;
const IRON: u16 = 25;

fn manual_save_market() -> GoodsMarket {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/saves/sweden_1837_08_16.v3"
    );
    let save = std::fs::read(path).expect("the manual Save fixture is readable");
    read_save(&save, &Rakaly::on_path())
        .expect("the manual Save is read")
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
    match &observation.value {
        Value::Read(value) => *value,
        Value::Missing { reason } => panic!("expected a read value, got missing: {reason}"),
    }
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
    assert_close(read_value(&clothes.buy_orders_from_buildings), 0.0);
    let tanks = good(&market, TANKS);
    assert_close(read_value(&tanks.sell_orders_from_buildings), 0.0);
    assert_close(read_value(&tanks.buy_orders_from_buildings), 0.0);
}
