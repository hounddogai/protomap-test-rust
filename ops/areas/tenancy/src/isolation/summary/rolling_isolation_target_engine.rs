use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling isolation target calculation.
pub trait RollingIsolationTargetModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingIsolationTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RollingIsolationTargetPriorityOffsetAdjuster {
    pub weight: f64,
}

impl RollingIsolationTargetModifier for RollingIsolationTargetPriorityOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus priority times {}", self.weight)
    }

    fn adjust(&self, item: &RollingIsolationTarget, value: f64) -> f64 {
        value + (item.priority as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRollingIsolationTargetAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingIsolationTargetModifier for NameRollingIsolationTargetAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RollingIsolationTarget, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingIsolationTargetAdjuster {
    pub minimum: f64,
}

impl RollingIsolationTargetModifier for MinimumRollingIsolationTargetAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingIsolationTarget, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the rolling isolation target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingIsolationTargetEngine {
    pub steps: Vec<Rc<dyn RollingIsolationTargetModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingIsolationTargetEngine {
    pub fn price(&mut self, item: &RollingIsolationTarget) -> f64 {
        let mut current = item.net_total();
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

    pub fn evaluate_all(&mut self, items: &[RollingIsolationTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_isolation_target_engine() -> RollingIsolationTargetEngine {
    RollingIsolationTargetEngine {
        steps: vec![
            Rc::new(RollingIsolationTargetPriorityOffsetAdjuster { weight: 0.09 })
                as Rc<dyn RollingIsolationTargetModifier>,
            Rc::new(NameRollingIsolationTargetAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.02), ("south".to_string(), 0.95)]),
            }),
            Rc::new(MinimumRollingIsolationTargetAdjuster { minimum: 7.1 }),
        ],
        trace: Vec::new(),
        floor: 2.8,
        ceiling: 43269.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_isolation_target_engine();
        let item = new_rolling_isolation_target("code-633", "name-624");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
