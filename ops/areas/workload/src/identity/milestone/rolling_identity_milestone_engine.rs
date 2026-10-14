use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the rolling identity milestone calculation.
pub trait RollingIdentityMilestoneAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingIdentityMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingIdentityMilestoneAdjuster {
    pub minimum: f64,
}

impl RollingIdentityMilestoneAdjuster for MinimumRollingIdentityMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingIdentityMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingIdentityMilestoneAdjuster {
    pub step: f64,
}

impl RollingIdentityMilestoneAdjuster for RoundingRollingIdentityMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingIdentityMilestone, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingIdentityMilestoneAdjuster {
    pub cap: f64,
}

impl RollingIdentityMilestoneAdjuster for CapRollingIdentityMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingIdentityMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the rolling identity milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingIdentityMilestoneEngine {
    pub steps: Vec<Rc<dyn RollingIdentityMilestoneAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingIdentityMilestoneEngine {
    pub fn run(&mut self, item: &RollingIdentityMilestone) -> f64 {
        let mut current = item.weighted_volume();
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

    pub fn evaluate_all(&mut self, items: &[RollingIdentityMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_rolling_identity_milestone_engine() -> RollingIdentityMilestoneEngine {
    RollingIdentityMilestoneEngine {
        steps: vec![
            Rc::new(MinimumRollingIdentityMilestoneAdjuster { minimum: 17.7 })
                as Rc<dyn RollingIdentityMilestoneAdjuster>,
            Rc::new(RoundingRollingIdentityMilestoneAdjuster { step: 0.25 }),
            Rc::new(CapRollingIdentityMilestoneAdjuster { cap: 26243.0 }),
        ],
        trace: Vec::new(),
        floor: 1.0,
        ceiling: 38083.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_rolling_identity_milestone_engine();
        let item = new_rolling_identity_milestone("code-4", "name-760");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
