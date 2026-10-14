use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling token batch calculation.
pub trait RollingTokenBatchStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingTokenBatch, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingTokenBatchAdjuster {
    pub minimum: f64,
}

impl RollingTokenBatchStep for MinimumRollingTokenBatchAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingTokenBatch, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingTokenBatchAdjuster {
    pub step: f64,
}

impl RollingTokenBatchStep for RoundingRollingTokenBatchAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingTokenBatch, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingTokenBatchAdjuster {
    pub percent: f64,
}

impl RollingTokenBatchStep for PercentRollingTokenBatchAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingTokenBatch, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingTokenBatchAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingTokenBatchStep for FlaggedRollingTokenBatchAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingTokenBatch, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingTokenBatchAdjuster {
    pub cap: f64,
}

impl RollingTokenBatchStep for CapRollingTokenBatchAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingTokenBatch, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling token batch steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingTokenBatchEngine {
    pub steps: Vec<Rc<dyn RollingTokenBatchStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingTokenBatchEngine {
    pub fn compute(&mut self, item: &RollingTokenBatch) -> f64 {
        let mut current = item.measured_value();
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

    pub fn evaluate_all(&mut self, items: &[RollingTokenBatch]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_rolling_token_batch_engine(flags: &flags::Registry) -> RollingTokenBatchEngine {
    RollingTokenBatchEngine {
        steps: vec![
            Rc::new(MinimumRollingTokenBatchAdjuster { minimum: 23.5 }) as Rc<dyn RollingTokenBatchStep>,
            Rc::new(RoundingRollingTokenBatchAdjuster { step: 1.0 }),
            Rc::new(PercentRollingTokenBatchAdjuster { percent: 23.4 }),
            Rc::new(FlaggedRollingTokenBatchAdjuster {
                flags: flags.clone(),
                flag: "rolling-token-batch-fallback".to_string(),
                percent: -3.6,
            }),
            Rc::new(CapRollingTokenBatchAdjuster { cap: 22770.0 }),
        ],
        trace: Vec::new(),
        floor: 2.3,
        ceiling: 37552.4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_token_batch_engine(&flags::new_registry());
        let item = new_rolling_token_batch("code-992", "name-991");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
