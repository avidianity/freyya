use serde::Serialize;

use crate::observation::{Observation, Unit};
use crate::save::SaveBudget;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Money {
    pub treasury: Observation,
    pub weekly_income: Observation,
    pub weekly_expenses: Observation,
    pub debt: Observation,
}

impl Money {
    pub(crate) fn from_budget(country_id: u32, budget: Option<&SaveBudget>) -> Self {
        let empty = SaveBudget::default();
        let budget = budget.unwrap_or(&empty);
        let source = |field: &str| format!("country_manager.database.{country_id}.budget.{field}");
        Money {
            treasury: Observation::of(budget.money, Unit::Pounds, source("money")),
            weekly_income: Observation::of(
                sum(&budget.weekly_income),
                Unit::PoundsPerWeek,
                source("weekly_income (sum)"),
            ),
            weekly_expenses: Observation::of(
                sum(&budget.weekly_expenses),
                Unit::PoundsPerWeek,
                source("weekly_expenses (sum)"),
            ),
            debt: Observation::of(budget.principal, Unit::Pounds, source("principal")),
        }
    }
}

fn sum(lines: &[f64]) -> Option<f64> {
    if lines.is_empty() {
        None
    } else {
        Some(lines.iter().sum())
    }
}
