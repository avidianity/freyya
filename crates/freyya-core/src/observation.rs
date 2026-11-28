use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Observation<T> {
    pub value: Option<T>,
    pub unit: Unit,
    pub source: String,
}

impl<T> Observation<T> {
    pub(crate) fn new(value: Option<T>, unit: Unit, source: String) -> Self {
        Observation {
            value,
            unit,
            source,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unit {
    ConstructionPoints,
    ConstructionPointsPerWeek,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Unit::ConstructionPoints => "construction points",
            Unit::ConstructionPointsPerWeek => "construction points per week",
        })
    }
}

impl<T: fmt::Display> fmt::Display for Observation<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.value {
            Some(value) => write!(f, "{value} {} [{}]", self.unit, self.source),
            None => write!(f, "missing ({}) [{}]", self.unit, self.source),
        }
    }
}
