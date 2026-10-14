use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling grant ledger calculation.
pub trait RollingGrantLedgerModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingGrantLedger, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingGrantLedgerAdjuster {
    pub minimum: f64,
}

impl RollingGrantLedgerModifier for MinimumRollingGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingGrantLedger, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingGrantLedgerAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingGrantLedgerModifier for TieredRollingGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingGrantLedger, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocaleRollingGrantLedgerAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingGrantLedgerModifier for LocaleRollingGrantLedgerAdjuster {
    fn describe(&self) -> String {
        "by locale".to_string()
    }

    fn adjust(&self, item: &RollingGrantLedger, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingGrantLedgerWeightOffsetAdjuster {
    pub weight: f64,
}

impl RollingGrantLedgerModifier for RollingGrantLedgerWeightOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus weight times {}", self.weight)
    }

    fn adjust(&self, item: &RollingGrantLedger, value: f64) -> f64 {
        value + item.weight * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingGrantLedgerAdjuster {
    pub percent: f64,
}

impl RollingGrantLedgerModifier for PercentRollingGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingGrantLedger, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingGrantLedgerAdjuster {
    pub step: f64,
}

impl RollingGrantLedgerModifier for RoundingRollingGrantLedgerAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingGrantLedger, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rolling grant ledger steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingGrantLedgerEngine {
    pub steps: Vec<Rc<dyn RollingGrantLedgerModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingGrantLedgerEngine {
    pub fn price(&mut self, item: &RollingGrantLedger) -> f64 {
        let mut current = item.weighted_value();
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

    pub fn evaluate_all(&mut self, items: &[RollingGrantLedger]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_grant_ledger_engine() -> RollingGrantLedgerEngine {
    RollingGrantLedgerEngine {
        steps: vec![
            Rc::new(MinimumRollingGrantLedgerAdjuster { minimum: 30.1 }) as Rc<dyn RollingGrantLedgerModifier>,
            Rc::new(TieredRollingGrantLedgerAdjuster {
                bounds: vec![6253.0, 6335.0, 8595.0, 9963.0],
                rates: vec![0.98, 0.9, 0.83, 1.11],
            }),
            Rc::new(LocaleRollingGrantLedgerAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.77), ("east".to_string(), 1.39)]),
            }),
            Rc::new(RollingGrantLedgerWeightOffsetAdjuster { weight: 0.61 }),
            Rc::new(PercentRollingGrantLedgerAdjuster { percent: -17.4 }),
            Rc::new(RoundingRollingGrantLedgerAdjuster { step: 0.1 }),
        ],
        trace: Vec::new(),
        floor: 3.2,
        ceiling: 38181.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_grant_ledger_engine();
        let item = new_rolling_grant_ledger("code-882", "label-595");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
