use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the default plan policy calculation.
pub trait DefaultPlanPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &DefaultPlanPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct RegionDefaultPlanPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl DefaultPlanPolicyAdjuster for RegionDefaultPlanPolicyAdjuster {
    fn describe(&self) -> String {
        "by region".to_string()
    }

    fn adjust(&self, item: &DefaultPlanPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumDefaultPlanPolicyAdjuster {
    pub minimum: f64,
}

impl DefaultPlanPolicyAdjuster for MinimumDefaultPlanPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &DefaultPlanPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedDefaultPlanPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl DefaultPlanPolicyAdjuster for FlaggedDefaultPlanPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &DefaultPlanPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapDefaultPlanPolicyAdjuster {
    pub cap: f64,
}

impl DefaultPlanPolicyAdjuster for CapDefaultPlanPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &DefaultPlanPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingDefaultPlanPolicyAdjuster {
    pub step: f64,
}

impl DefaultPlanPolicyAdjuster for RoundingDefaultPlanPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &DefaultPlanPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the default plan policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct DefaultPlanPolicyEngine {
    pub steps: Vec<Rc<dyn DefaultPlanPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl DefaultPlanPolicyEngine {
    pub fn run(&mut self, item: &DefaultPlanPolicy) -> f64 {
        let mut current = item.measured_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn DefaultPlanPolicyAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[DefaultPlanPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_default_plan_policy_engine(flags: &flags::Registry) -> DefaultPlanPolicyEngine {
    DefaultPlanPolicyEngine {
        steps: vec![
            Rc::new(RegionDefaultPlanPolicyAdjuster {
                factors: BTreeMap::from([("north".to_string(), 0.82)]),
            }) as Rc<dyn DefaultPlanPolicyAdjuster>,
            Rc::new(MinimumDefaultPlanPolicyAdjuster { minimum: 6.1 }),
            Rc::new(FlaggedDefaultPlanPolicyAdjuster {
                flags: flags.clone(),
                flag: "default-plan-policy-legacy".to_string(),
                percent: 5.4,
            }),
            Rc::new(CapDefaultPlanPolicyAdjuster { cap: 41073.0 }),
            Rc::new(RoundingDefaultPlanPolicyAdjuster { step: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 0.7,
        ceiling: 84461.2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_default_plan_policy_engine(&flags::new_registry());
        let item = new_default_plan_policy("code-601", "name-653");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
