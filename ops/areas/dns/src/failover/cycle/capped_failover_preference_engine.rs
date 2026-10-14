use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the capped failover preference calculation.
pub trait CappedFailoverPreferenceModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &CappedFailoverPreference, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingCappedFailoverPreferenceAdjuster {
    pub step: f64,
}

impl CappedFailoverPreferenceModifier for RoundingCappedFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &CappedFailoverPreference, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredCappedFailoverPreferenceAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl CappedFailoverPreferenceModifier for TieredCappedFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &CappedFailoverPreference, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumCappedFailoverPreferenceAdjuster {
    pub minimum: f64,
}

impl CappedFailoverPreferenceModifier for MinimumCappedFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &CappedFailoverPreference, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentCappedFailoverPreferenceAdjuster {
    pub percent: f64,
}

impl CappedFailoverPreferenceModifier for PercentCappedFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &CappedFailoverPreference, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedCappedFailoverPreferenceAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl CappedFailoverPreferenceModifier for FlaggedCappedFailoverPreferenceAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &CappedFailoverPreference, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the capped failover preference steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct CappedFailoverPreferenceEngine {
    pub steps: Vec<Rc<dyn CappedFailoverPreferenceModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl CappedFailoverPreferenceEngine {
    pub fn evaluate(&mut self, item: &CappedFailoverPreference) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn CappedFailoverPreferenceModifier> = self.steps[index as usize].clone();
            let before = current;
            current = step.adjust(item, current);
            if before != current {
                self.trace.push(format!(
                    "{}. {} changed {} to {}",
                    index + 1,
                    step.describe(),
                    before,
                    current
                ));
            }
        }
        if current < self.floor {
            return self.floor;
        }
        if current > self.ceiling {
            return self.ceiling;
        }
        current
    }

    pub fn explain(&self) -> String {
        self.trace.join("\n")
    }

    pub fn evaluate_all(&mut self, items: &[CappedFailoverPreference]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_capped_failover_preference_engine(flags: &flags::Registry) -> CappedFailoverPreferenceEngine {
    CappedFailoverPreferenceEngine {
        steps: vec![
            Rc::new(RoundingCappedFailoverPreferenceAdjuster { step: 5.0 }) as Rc<dyn CappedFailoverPreferenceModifier>,
            Rc::new(TieredCappedFailoverPreferenceAdjuster {
                bounds: vec![796.0, 8563.0, 9551.0, 9887.0],
                rates: vec![0.84, 0.88, 0.87, 0.89],
            }),
            Rc::new(MinimumCappedFailoverPreferenceAdjuster { minimum: 44.4 }),
            Rc::new(PercentCappedFailoverPreferenceAdjuster { percent: -20.8 }),
            Rc::new(FlaggedCappedFailoverPreferenceAdjuster {
                flags: flags.clone(),
                flag: "capped-failover-preference-strict".to_string(),
                percent: 8.9,
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 26874.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_capped_failover_preference_engine(&flags::new_registry());
        let item = new_capped_failover_preference("code-311", "name-322");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
