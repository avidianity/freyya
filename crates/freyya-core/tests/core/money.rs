use freyya_core::{Observation, Reading, Unit};

use crate::helpers::{player_save, read_manual_save, read_text_save};

fn text_save_with_player_country(country: &str) -> Vec<u8> {
    player_save(country, "")
}

fn assert_read(observation: &Observation, expected: f64, unit: Unit, source: &str) {
    match observation.reading {
        Reading::Read(value) => assert!(
            (value - expected).abs() < 1e-6,
            "{source}: read {value}, expected {expected}"
        ),
        Reading::Missing { .. } => panic!("{source}: missing, expected {expected}"),
    }
    assert_eq!(observation.unit, unit);
    assert_eq!(observation.source, source);
}

fn assert_missing(observation: &Observation, unit: Unit, source: &str) {
    assert_eq!(
        observation.reading,
        Reading::Missing { reason: None },
        "{source}"
    );
    assert_eq!(observation.unit, unit);
    assert_eq!(observation.source, source);
}

#[test]
fn manual_save_gives_treasury_income_and_expenses_and_marks_absent_debt_missing() {
    let snapshot = read_manual_save();
    let money = snapshot.money;

    assert_read(
        &money.treasury,
        355998.7373,
        Unit::Pounds,
        "country_manager.database.24.budget.money",
    );
    assert_read(
        &money.weekly_income,
        22482.04448,
        Unit::PoundsPerWeek,
        "country_manager.database.24.budget.weekly_income (sum)",
    );
    assert_read(
        &money.weekly_expenses,
        18602.32286,
        Unit::PoundsPerWeek,
        "country_manager.database.24.budget.weekly_expenses (sum)",
    );
    assert_missing(
        &money.debt,
        Unit::Pounds,
        "country_manager.database.24.budget.principal",
    );
}

#[test]
fn country_in_debt_gives_its_principal_as_debt() {
    let save = text_save_with_player_country(
        "budget={ money=0 principal=1860613.51589 weekly_income={ 10 20.5 } weekly_expenses={ 7 } }",
    );

    let money = read_text_save(&save).money;

    assert_read(
        &money.debt,
        1860613.51589,
        Unit::Pounds,
        "country_manager.database.7.budget.principal",
    );
    assert_read(
        &money.treasury,
        0.0,
        Unit::Pounds,
        "country_manager.database.7.budget.money",
    );
    assert_read(
        &money.weekly_income,
        30.5,
        Unit::PoundsPerWeek,
        "country_manager.database.7.budget.weekly_income (sum)",
    );
}

#[test]
fn country_without_a_budget_marks_every_money_observation_missing() {
    let save = text_save_with_player_country("");

    let money = read_text_save(&save).money;

    assert_missing(
        &money.treasury,
        Unit::Pounds,
        "country_manager.database.7.budget.money",
    );
    assert_missing(
        &money.weekly_income,
        Unit::PoundsPerWeek,
        "country_manager.database.7.budget.weekly_income (sum)",
    );
    assert_missing(
        &money.weekly_expenses,
        Unit::PoundsPerWeek,
        "country_manager.database.7.budget.weekly_expenses (sum)",
    );
    assert_missing(
        &money.debt,
        Unit::Pounds,
        "country_manager.database.7.budget.principal",
    );
}

#[test]
fn empty_income_and_expense_lists_are_missing_not_zero() {
    let save = text_save_with_player_country(
        "budget={ money=12.5 weekly_income={ } weekly_expenses={ } }",
    );

    let money = read_text_save(&save).money;

    assert_read(
        &money.treasury,
        12.5,
        Unit::Pounds,
        "country_manager.database.7.budget.money",
    );
    assert_missing(
        &money.weekly_income,
        Unit::PoundsPerWeek,
        "country_manager.database.7.budget.weekly_income (sum)",
    );
    assert_missing(
        &money.weekly_expenses,
        Unit::PoundsPerWeek,
        "country_manager.database.7.budget.weekly_expenses (sum)",
    );
}
