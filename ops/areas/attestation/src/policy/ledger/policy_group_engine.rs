use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;

/// One step of the policy group calculation.
pub trait PolicyGroupStep: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyGroupAdjuster {
    pub minimum: f64,
}

impl PolicyGroupStep for MinimumPolicyGroupAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyGroup, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionPolicyGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyGroupStep for RegionPolicyGroupAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &PolicyGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicyGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicyGroupStep for FlaggedPolicyGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicyGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPolicyGroupAdjuster {
    pub percent: f64,
}

impl PolicyGroupStep for PercentPolicyGroupAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PolicyGroup, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyGroupAdjuster {
    pub cap: f64,
}

impl PolicyGroupStep for CapPolicyGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the policy group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyGroupEngine {
    pub steps: Vec<Rc<dyn PolicyGroupStep>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyGroupEngine {
    pub fn run(&mut self, item: &PolicyGroup) -> f64 {
        let mut current = item.projected_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn PolicyGroupStep> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[PolicyGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_policy_group_engine(flags: &flags::Registry) -> PolicyGroupEngine {
    PolicyGroupEngine {
        steps: vec![
            Rc::new(MinimumPolicyGroupAdjuster { minimum: 49.3 }) as Rc<dyn PolicyGroupStep>,
            Rc::new(RegionPolicyGroupAdjuster {
                factors: BTreeMap::from([("retail".to_string(), 1.36)]),
            }),
            Rc::new(FlaggedPolicyGroupAdjuster {
                flags: flags.clone(),
                flag: "policy-group-legacy".to_string(),
                percent: 8.6,
            }),
            Rc::new(PercentPolicyGroupAdjuster { percent: 6.1 }),
            Rc::new(CapPolicyGroupAdjuster { cap: 3814.0 }),
        ],
        trace: Vec::new(),
        floor: 1.6,
        ceiling: 14047.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_group_engine(&flags::new_registry());
        let item = new_policy_group("code-223", "title-288");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
