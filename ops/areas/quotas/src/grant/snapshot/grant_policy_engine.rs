use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::numbers;

/// One step of the grant policy calculation.
pub trait GrantPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GrantPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentGrantPolicyAdjuster {
    pub percent: f64,
}

impl GrantPolicyAdjuster for PercentGrantPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &GrantPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGrantPolicyAdjuster {
    pub minimum: f64,
}

impl GrantPolicyAdjuster for MinimumGrantPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GrantPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterGrantPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GrantPolicyAdjuster for ClusterGrantPolicyAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &GrantPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGrantPolicyAdjuster {
    pub step: f64,
}

impl GrantPolicyAdjuster for RoundingGrantPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GrantPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GrantPolicyCeilingOffsetAdjuster {
    pub weight: f64,
}

impl GrantPolicyAdjuster for GrantPolicyCeilingOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus ceiling times {}", self.weight)
    }

    fn adjust(&self, item: &GrantPolicy, value: f64) -> f64 {
        value + item.ceiling * self.weight
    }
}

/// Runs the grant policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GrantPolicyEngine {
    pub steps: Vec<Rc<dyn GrantPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GrantPolicyEngine {
    pub fn run(&mut self, item: &GrantPolicy) -> f64 {
        let mut current = item.effective_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn GrantPolicyAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[GrantPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_grant_policy_engine() -> GrantPolicyEngine {
    GrantPolicyEngine {
        steps: vec![
            Rc::new(PercentGrantPolicyAdjuster { percent: -1.8 }) as Rc<dyn GrantPolicyAdjuster>,
            Rc::new(MinimumGrantPolicyAdjuster { minimum: 43.7 }),
            Rc::new(ClusterGrantPolicyAdjuster {
                factors: BTreeMap::from([("central".to_string(), 1.01)]),
            }),
            Rc::new(RoundingGrantPolicyAdjuster { step: 0.25 }),
            Rc::new(GrantPolicyCeilingOffsetAdjuster { weight: 1.8 }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 44877.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_grant_policy_engine();
        let item = new_grant_policy("code-95", "name-898");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
