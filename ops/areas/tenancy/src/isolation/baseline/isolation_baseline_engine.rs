use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the isolation baseline calculation.
pub trait IsolationBaselineAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IsolationBaseline, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIsolationBaselineAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IsolationBaselineAdjuster for TieredIsolationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IsolationBaseline, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingIsolationBaselineAdjuster {
    pub step: f64,
}

impl IsolationBaselineAdjuster for RoundingIsolationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &IsolationBaseline, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIsolationBaselineAdjuster {
    pub percent: f64,
}

impl IsolationBaselineAdjuster for PercentIsolationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IsolationBaseline, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIsolationBaselineAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IsolationBaselineAdjuster for FlaggedIsolationBaselineAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IsolationBaseline, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the isolation baseline steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IsolationBaselineEngine {
    pub steps: Vec<Rc<dyn IsolationBaselineAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IsolationBaselineEngine {
    pub fn evaluate(&mut self, item: &IsolationBaseline) -> f64 {
        let mut current = item.effective_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn IsolationBaselineAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[IsolationBaseline]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_isolation_baseline_engine(flags: &flags::Registry) -> IsolationBaselineEngine {
    IsolationBaselineEngine {
        steps: vec![
            Rc::new(TieredIsolationBaselineAdjuster {
                bounds: vec![6407.0, 6876.0],
                rates: vec![1.08, 0.95],
            }) as Rc<dyn IsolationBaselineAdjuster>,
            Rc::new(RoundingIsolationBaselineAdjuster { step: 0.05 }),
            Rc::new(PercentIsolationBaselineAdjuster { percent: 22.3 }),
            Rc::new(FlaggedIsolationBaselineAdjuster {
                flags: flags.clone(),
                flag: "isolation-baseline-capped".to_string(),
                percent: -7.3,
            }),
        ],
        trace: Vec::new(),
        floor: 4.2,
        ceiling: 43279.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_isolation_baseline_engine(&flags::new_registry());
        let item = new_isolation_baseline("code-494", "label-303");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
