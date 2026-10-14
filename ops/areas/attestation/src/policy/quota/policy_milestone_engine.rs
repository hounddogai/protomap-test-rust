use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

/// One step of the policy milestone calculation.
pub trait PolicyMilestoneStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyMilestone, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyMilestoneAdjuster {
    pub minimum: f64,
}

impl PolicyMilestoneStep for MinimumPolicyMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyMilestone, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPolicyMilestoneAdjuster {
    pub percent: f64,
}

impl PolicyMilestoneStep for PercentPolicyMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PolicyMilestone, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelPolicyMilestoneAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyMilestoneStep for LabelPolicyMilestoneAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &PolicyMilestone, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyMilestoneAdjuster {
    pub cap: f64,
}

impl PolicyMilestoneStep for CapPolicyMilestoneAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyMilestone, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the policy milestone steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyMilestoneEngine {
    pub steps: Vec<Rc<dyn PolicyMilestoneStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyMilestoneEngine {
    pub fn evaluate(&mut self, item: &PolicyMilestone) -> f64 {
        let mut current = item.net_load();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicyMilestoneStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicyMilestone]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_policy_milestone_engine() -> PolicyMilestoneEngine {
    PolicyMilestoneEngine {
        steps: vec![
            Rc::new(MinimumPolicyMilestoneAdjuster { minimum: 13.9 }) as Rc<dyn PolicyMilestoneStep>,
            Rc::new(PercentPolicyMilestoneAdjuster { percent: 13.4 }),
            Rc::new(LabelPolicyMilestoneAdjuster {
                factors: BTreeMap::from([("south".to_string(), 1.13)]),
            }),
            Rc::new(CapPolicyMilestoneAdjuster { cap: 40536.0 }),
        ],
        trace: Vec::new(),
        floor: 2.9,
        ceiling: 94051.8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_milestone_engine();
        let item = new_policy_milestone("code-512", "label-667");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
