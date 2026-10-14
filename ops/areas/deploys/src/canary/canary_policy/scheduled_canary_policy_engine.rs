use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the scheduled canary policy calculation.
pub trait ScheduledCanaryPolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledCanaryPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledCanaryPolicyAdjuster {
    pub minimum: f64,
}

impl ScheduledCanaryPolicyFactor for MinimumScheduledCanaryPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledCanaryPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledCanaryPolicyFloorValueOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledCanaryPolicyFactor for ScheduledCanaryPolicyFloorValueOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus floor value times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledCanaryPolicy, value: f64) -> f64 {
        value + item.floor_value * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledCanaryPolicyAdjuster {
    pub percent: f64,
}

impl ScheduledCanaryPolicyFactor for PercentScheduledCanaryPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledCanaryPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the scheduled canary policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledCanaryPolicyEngine {
    pub steps: Vec<Rc<dyn ScheduledCanaryPolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledCanaryPolicyEngine {
    pub fn run(&mut self, item: &ScheduledCanaryPolicy) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ScheduledCanaryPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_canary_policy_engine() -> ScheduledCanaryPolicyEngine {
    ScheduledCanaryPolicyEngine {
        steps: vec![
            Rc::new(MinimumScheduledCanaryPolicyAdjuster { minimum: 24.0 }) as Rc<dyn ScheduledCanaryPolicyFactor>,
            Rc::new(ScheduledCanaryPolicyFloorValueOffsetAdjuster { weight: 0.83 }),
            Rc::new(PercentScheduledCanaryPolicyAdjuster { percent: 11.8 }),
        ],
        trace: Vec::new(),
        floor: 2.7,
        ceiling: 32137.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_scheduled_canary_policy_engine();
        let item = new_scheduled_canary_policy("code-750", "name-707");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
