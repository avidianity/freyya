use serde::{Deserialize, Serialize};

use crate::goods_market::GoodAmounts;
use crate::observation::{Observation, Unit, observe};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Money {
    pub treasury: Observation,
    pub weekly_income: Observation,
    pub weekly_expenses: Observation,
    pub debt: Observation,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct Budget {
    #[serde(default)]
    money: Option<f64>,
    #[serde(default)]
    weekly_income: Vec<f64>,
    #[serde(default)]
    weekly_expenses: Vec<f64>,
    #[serde(default)]
    principal: Option<f64>,
    #[serde(default)]
    pub(crate) current_price_report: Option<GoodAmounts>,
}

impl Money {
    pub(crate) fn from_budget(country_id: u32, budget: Option<&Budget>) -> Self {
        let empty = Budget::default();
        let budget = budget.unwrap_or(&empty);
        let source = |field: &str| format!("country_manager.database.{country_id}.budget.{field}");
        Money {
            treasury: observe(budget.money, Unit::Pounds, source("money")),
            weekly_income: observe(
                sum(&budget.weekly_income),
                Unit::PoundsPerWeek,
                source("weekly_income (sum)"),
            ),
            weekly_expenses: observe(
                sum(&budget.weekly_expenses),
                Unit::PoundsPerWeek,
                source("weekly_expenses (sum)"),
            ),
            debt: observe(budget.principal, Unit::Pounds, source("principal")),
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
