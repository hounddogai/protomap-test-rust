use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling postmortem policy calculation.
pub trait RollingPostmortemPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingPostmortemPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingPostmortemPolicyAdjuster {
    pub percent: f64,
}

impl RollingPostmortemPolicyAdjuster for PercentRollingPostmortemPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingPostmortemPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingPostmortemPolicyAdjuster {
    pub minimum: f64,
}

impl RollingPostmortemPolicyAdjuster for MinimumRollingPostmortemPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingPostmortemPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredRollingPostmortemPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl RollingPostmortemPolicyAdjuster for TieredRollingPostmortemPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &RollingPostmortemPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the rolling postmortem policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingPostmortemPolicyEngine {
    pub steps: Vec<Rc<dyn RollingPostmortemPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingPostmortemPolicyEngine {
    pub fn run(&mut self, item: &RollingPostmortemPolicy) -> f64 {
        let mut current = item.net_load();
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

    pub fn evaluate_all(&mut self, items: &[RollingPostmortemPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_postmortem_policy_engine() -> RollingPostmortemPolicyEngine {
    RollingPostmortemPolicyEngine {
        steps: vec![
            Rc::new(PercentRollingPostmortemPolicyAdjuster { percent: 2.5 }) as Rc<dyn RollingPostmortemPolicyAdjuster>,
            Rc::new(MinimumRollingPostmortemPolicyAdjuster { minimum: 8.8 }),
            Rc::new(TieredRollingPostmortemPolicyAdjuster {
                bounds: vec![4667.0, 7791.0, 8907.0],
                rates: vec![1.19, 1.1, 1.09],
            }),
        ],
        trace: Vec::new(),
        floor: 0.4,
        ceiling: 60990.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_postmortem_policy_engine();
        let item = new_rolling_postmortem_policy("code-389", "label-598");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
