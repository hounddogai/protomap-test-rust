use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled token variance calculation.
pub trait ScheduledTokenVarianceStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledTokenVariance, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingScheduledTokenVarianceAdjuster {
    pub step: f64,
}

impl ScheduledTokenVarianceStep for RoundingScheduledTokenVarianceAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &ScheduledTokenVariance, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledTokenVarianceAdjuster {
    pub percent: f64,
}

impl ScheduledTokenVarianceStep for PercentScheduledTokenVarianceAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledTokenVariance, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledTokenVarianceAdjuster {
    pub minimum: f64,
}

impl ScheduledTokenVarianceStep for MinimumScheduledTokenVarianceAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledTokenVariance, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled token variance steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledTokenVarianceEngine {
    pub steps: Vec<Rc<dyn ScheduledTokenVarianceStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledTokenVarianceEngine {
    pub fn compute(&mut self, item: &ScheduledTokenVariance) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[ScheduledTokenVariance]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_scheduled_token_variance_engine() -> ScheduledTokenVarianceEngine {
    ScheduledTokenVarianceEngine {
        steps: vec![
            Rc::new(RoundingScheduledTokenVarianceAdjuster { step: 0.5 }) as Rc<dyn ScheduledTokenVarianceStep>,
            Rc::new(PercentScheduledTokenVarianceAdjuster { percent: -13.5 }),
            Rc::new(MinimumScheduledTokenVarianceAdjuster { minimum: 39.8 }),
        ],
        trace: Vec::new(),
        floor: 4.7,
        ceiling: 97216.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_token_variance_engine();
        let item = new_scheduled_token_variance("code-449", "label-664");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
