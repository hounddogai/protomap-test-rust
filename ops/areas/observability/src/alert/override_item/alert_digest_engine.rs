use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the alert digest calculation.
pub trait AlertDigestFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AlertDigest, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAlertDigestAdjuster {
    pub step: f64,
}

impl AlertDigestFactor for RoundingAlertDigestAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AlertDigest, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAlertDigestAdjuster {
    pub minimum: f64,
}

impl AlertDigestFactor for MinimumAlertDigestAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AlertDigest, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAlertDigestAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AlertDigestFactor for TieredAlertDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AlertDigest, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AlertDigestSequenceOffsetAdjuster {
    pub weight: f64,
}

impl AlertDigestFactor for AlertDigestSequenceOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus sequence times {}", self.weight)
    }

    fn adjust(&self, item: &AlertDigest, value: f64) -> f64 {
        value + (item.sequence as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAlertDigestAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AlertDigestFactor for FlaggedAlertDigestAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AlertDigest, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the alert digest steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AlertDigestEngine {
    pub steps: Vec<Rc<dyn AlertDigestFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AlertDigestEngine {
    pub fn evaluate(&mut self, item: &AlertDigest) -> f64 {
        let mut current = item.projected_exposure();
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

    pub fn evaluate_all(&mut self, items: &[AlertDigest]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_alert_digest_engine(flags: &flags::Registry) -> AlertDigestEngine {
    AlertDigestEngine {
        steps: vec![
            Rc::new(RoundingAlertDigestAdjuster { step: 0.25 }) as Rc<dyn AlertDigestFactor>,
            Rc::new(MinimumAlertDigestAdjuster { minimum: 18.9 }),
            Rc::new(TieredAlertDigestAdjuster {
                bounds: vec![4320.0, 8292.0],
                rates: vec![0.96, 0.85],
            }),
            Rc::new(AlertDigestSequenceOffsetAdjuster { weight: 1.9 }),
            Rc::new(FlaggedAlertDigestAdjuster {
                flags: flags.clone(),
                flag: "alert-digest-tiered".to_string(),
                percent: 8.7,
            }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 16243.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_alert_digest_engine(&flags::new_registry());
        let item = new_alert_digest("code-350", "label-712");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
