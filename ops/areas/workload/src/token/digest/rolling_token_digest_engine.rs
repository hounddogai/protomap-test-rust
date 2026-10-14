use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling token digest calculation.
pub trait RollingTokenDigestModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingTokenDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct UnitRollingTokenDigestAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingTokenDigestModifier for UnitRollingTokenDigestAdjuster {
    fn describe(&self) -> String {
        "by unit".to_string()
    }

    fn adjust(&self, item: &RollingTokenDigest, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingTokenDigestAdjuster {
    pub cap: f64,
}

impl RollingTokenDigestModifier for CapRollingTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingTokenDigest, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingTokenDigestAdjuster {
    pub minimum: f64,
}

impl RollingTokenDigestModifier for MinimumRollingTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingTokenDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingTokenDigestDiscountOffsetAdjuster {
    pub weight: f64,
}

impl RollingTokenDigestModifier for RollingTokenDigestDiscountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus discount times {}", self.weight)
    }

    fn adjust(&self, item: &RollingTokenDigest, value: f64) -> f64 {
        value + item.discount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingTokenDigestAdjuster {
    pub percent: f64,
}

impl RollingTokenDigestModifier for PercentRollingTokenDigestAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingTokenDigest, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the rolling token digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingTokenDigestEngine {
    pub steps: Vec<Rc<dyn RollingTokenDigestModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingTokenDigestEngine {
    pub fn price(&mut self, item: &RollingTokenDigest) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[RollingTokenDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_token_digest_engine() -> RollingTokenDigestEngine {
    RollingTokenDigestEngine {
        steps: vec![
            Rc::new(UnitRollingTokenDigestAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.29), ("online".to_string(), 1.0)]),
            }) as Rc<dyn RollingTokenDigestModifier>,
            Rc::new(CapRollingTokenDigestAdjuster { cap: 41164.0 }),
            Rc::new(MinimumRollingTokenDigestAdjuster { minimum: 7.3 }),
            Rc::new(RollingTokenDigestDiscountOffsetAdjuster { weight: 1.68 }),
            Rc::new(PercentRollingTokenDigestAdjuster { percent: 10.1 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 55682.8,
    }
}
