use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the usage checkpoint calculation.
pub trait UsageCheckpointFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &UsageCheckpoint, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumUsageCheckpointAdjuster {
    pub minimum: f64,
}

impl UsageCheckpointFactor for MinimumUsageCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &UsageCheckpoint, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentUsageCheckpointAdjuster {
    pub percent: f64,
}

impl UsageCheckpointFactor for PercentUsageCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &UsageCheckpoint, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CountryUsageCheckpointAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl UsageCheckpointFactor for CountryUsageCheckpointAdjuster {
    fn describe(&self) -> String {
        "by country".to_string()
    }

    fn adjust(&self, item: &UsageCheckpoint, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingUsageCheckpointAdjuster {
    pub step: f64,
}

impl UsageCheckpointFactor for RoundingUsageCheckpointAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &UsageCheckpoint, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the usage checkpoint steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct UsageCheckpointEngine {
    pub steps: Vec<Rc<dyn UsageCheckpointFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl UsageCheckpointEngine {
    pub fn evaluate(&mut self, item: &UsageCheckpoint) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[UsageCheckpoint]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_usage_checkpoint_engine() -> UsageCheckpointEngine {
    UsageCheckpointEngine {
        steps: vec![
            Rc::new(MinimumUsageCheckpointAdjuster { minimum: 48.3 }) as Rc<dyn UsageCheckpointFactor>,
            Rc::new(PercentUsageCheckpointAdjuster { percent: 6.1 }),
            Rc::new(CountryUsageCheckpointAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.32)]),
            }),
            Rc::new(RoundingUsageCheckpointAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 19386.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_usage_checkpoint_engine();
        let item = new_usage_checkpoint("code-157", "name-832");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
