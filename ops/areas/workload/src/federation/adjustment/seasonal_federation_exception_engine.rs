use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the seasonal federation exception calculation.
pub trait SeasonalFederationExceptionModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &SeasonalFederationException, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalFederationExceptionLimitOffsetAdjuster {
    pub weight: f64,
}

impl SeasonalFederationExceptionModifier for SeasonalFederationExceptionLimitOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus limit times {}", self.weight)
    }

    fn adjust(&self, item: &SeasonalFederationException, value: f64) -> f64 {
        value + (item.limit as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedSeasonalFederationExceptionAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl SeasonalFederationExceptionModifier for FlaggedSeasonalFederationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &SeasonalFederationException, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumSeasonalFederationExceptionAdjuster {
    pub minimum: f64,
}

impl SeasonalFederationExceptionModifier for MinimumSeasonalFederationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &SeasonalFederationException, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountrySeasonalFederationExceptionAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl SeasonalFederationExceptionModifier for CountrySeasonalFederationExceptionAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &SeasonalFederationException, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingSeasonalFederationExceptionAdjuster {
    pub step: f64,
}

impl SeasonalFederationExceptionModifier for RoundingSeasonalFederationExceptionAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &SeasonalFederationException, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the seasonal federation exception steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct SeasonalFederationExceptionEngine {
    pub steps: Vec<Rc<dyn SeasonalFederationExceptionModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl SeasonalFederationExceptionEngine {
    pub fn run(&mut self, item: &SeasonalFederationException) -> f64 {
        let mut current = item.weighted_total();
        self.trace = Vec::new();
        for step in self.steps.iter() {
            let next = step.adjust(item, current);
            self.trace.push(format!(
                "{}: {} -> {}",
                step.describe(),
                (current * 100.0).round() / 100.0,
                (next * 100.0).round() / 100.0
            ));
            current = next;
        }
        numbers::clamp_float(current, self.floor, self.ceiling)
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[SeasonalFederationException]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_seasonal_federation_exception_engine(flags: &flags::Registry) -> SeasonalFederationExceptionEngine {
    SeasonalFederationExceptionEngine {
        steps: vec![
            Rc::new(SeasonalFederationExceptionLimitOffsetAdjuster { weight: 0.62 })
                as Rc<dyn SeasonalFederationExceptionModifier>,
            Rc::new(FlaggedSeasonalFederationExceptionAdjuster {
                flags: flags.clone(),
                flag: "seasonal-federation-exception-lenient".to_string(),
                percent: 3.8,
            }),
            Rc::new(MinimumSeasonalFederationExceptionAdjuster { minimum: 5.3 }),
            Rc::new(CountrySeasonalFederationExceptionAdjuster {
                factors: BTreeMap::from([
                    ("central".to_string(), 1.08),
                    ("retail".to_string(), 1.13),
                    ("east".to_string(), 1.33),
                ]),
            }),
            Rc::new(RoundingSeasonalFederationExceptionAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 3.8,
        ceiling: 99336.2,
    }
}
