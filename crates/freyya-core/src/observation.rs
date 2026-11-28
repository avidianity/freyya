use std::fmt;

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Observation<T = f64> {
    pub reading: Reading<T>,
    pub unit: Unit,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Reading<T = f64> {
    Read(T),
    Missing { reason: Option<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Unit {
    Pounds,
    PoundsPerWeek,
    PoundsPerUnit,
    ConstructionPoints,
    ConstructionPointsPerWeek,
    UnitsPerWeek,
    TradeVolume,
    People,
    Levels,
    StaffedLevels,
    ProductionMethods,
}

impl<T> Observation<T> {
    pub(crate) fn of(value: Option<T>, unit: Unit, source: String) -> Self {
        Observation {
            reading: value.map_or(Reading::Missing { reason: None }, Reading::Read),
            unit,
            source,
        }
    }

    pub(crate) fn missing(reason: impl Into<String>, unit: Unit, source: String) -> Self {
        Observation {
            reading: Reading::Missing {
                reason: Some(reason.into()),
            },
            unit,
            source,
        }
    }

    pub(crate) fn or_missing(
        value: Option<T>,
        reason: impl Into<String>,
        unit: Unit,
        source: String,
    ) -> Self {
        match value {
            Some(value) => Observation {
                reading: Reading::Read(value),
                unit,
                source,
            },
            None => Self::missing(reason, unit, source),
        }
    }
}

impl<T> Observation<T>
where
    Reading<T>: fmt::Display,
{
    pub fn shown(&self) -> String {
        match self.reading {
            Reading::Read(_) => format!("{} {}", self.reading, self.unit),
            Reading::Missing { .. } => self.reading.to_string(),
        }
    }
}

impl<T> fmt::Display for Observation<T>
where
    Reading<T>: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}  [{}]", self.shown(), self.source)
    }
}

fn write_reading<T>(
    reading: &Reading<T>,
    f: &mut fmt::Formatter<'_>,
    write_value: impl FnOnce(&T, &mut fmt::Formatter<'_>) -> fmt::Result,
) -> fmt::Result {
    match reading {
        Reading::Read(value) => write_value(value, f),
        Reading::Missing { reason: None } => f.write_str("missing"),
        Reading::Missing {
            reason: Some(reason),
        } => write!(f, "missing ({reason})"),
    }
}

impl fmt::Display for Reading<f64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_reading(self, f, |value, f| write!(f, "{value:.2}"))
    }
}

impl fmt::Display for Reading<u64> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_reading(self, f, |value, f| write!(f, "{value}"))
    }
}

impl fmt::Display for Reading<u32> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_reading(self, f, |value, f| write!(f, "{value}"))
    }
}

impl fmt::Display for Reading<Vec<String>> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write_reading(self, f, |values, f| f.write_str(&values.join(" ")))
    }
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Unit::Pounds => "£",
            Unit::PoundsPerWeek => "£/week",
            Unit::PoundsPerUnit => "£ per unit",
            Unit::ConstructionPoints => "construction points",
            Unit::ConstructionPointsPerWeek => "construction points/week",
            Unit::UnitsPerWeek => "units per week",
            Unit::TradeVolume => "trade volume",
            Unit::People => "people",
            Unit::Levels => "levels",
            Unit::StaffedLevels => "staffed levels",
            Unit::ProductionMethods => "production methods",
        })
    }
}
