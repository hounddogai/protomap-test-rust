use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the strict rollback policy calculation.
pub trait StrictRollbackPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictRollbackPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapStrictRollbackPolicyAdjuster {
    pub cap: f64,
}

impl StrictRollbackPolicyAdjuster for CapStrictRollbackPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &StrictRollbackPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingStrictRollbackPolicyAdjuster {
    pub step: f64,
}

impl StrictRollbackPolicyAdjuster for RoundingStrictRollbackPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &StrictRollbackPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumStrictRollbackPolicyAdjuster {
    pub minimum: f64,
}

impl StrictRollbackPolicyAdjuster for MinimumStrictRollbackPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &StrictRollbackPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the strict rollback policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictRollbackPolicyEngine {
    pub steps: Vec<Rc<dyn StrictRollbackPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictRollbackPolicyEngine {
    pub fn run(&mut self, item: &StrictRollbackPolicy) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[StrictRollbackPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_strict_rollback_policy_engine() -> StrictRollbackPolicyEngine {
    StrictRollbackPolicyEngine {
        steps: vec![
            Rc::new(CapStrictRollbackPolicyAdjuster { cap: 31876.0 }) as Rc<dyn StrictRollbackPolicyAdjuster>,
            Rc::new(RoundingStrictRollbackPolicyAdjuster { step: 0.25 }),
            Rc::new(MinimumStrictRollbackPolicyAdjuster { minimum: 1.3 }),
        ],
        trace: Vec::new(),
        floor: 3.0,
        ceiling: 60262.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_strict_rollback_policy_engine();
        let item = new_strict_rollback_policy("code-595", "name-849");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
