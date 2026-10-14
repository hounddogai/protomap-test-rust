use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the binding policy calculation.
pub trait BindingPolicyAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &BindingPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryBindingPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl BindingPolicyAdjuster for CategoryBindingPolicyAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &BindingPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentBindingPolicyAdjuster {
    pub percent: f64,
}

impl BindingPolicyAdjuster for PercentBindingPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &BindingPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedBindingPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl BindingPolicyAdjuster for FlaggedBindingPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &BindingPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingBindingPolicyAdjuster {
    pub step: f64,
}

impl BindingPolicyAdjuster for RoundingBindingPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &BindingPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumBindingPolicyAdjuster {
    pub minimum: f64,
}

impl BindingPolicyAdjuster for MinimumBindingPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &BindingPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredBindingPolicyAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl BindingPolicyAdjuster for TieredBindingPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &BindingPolicy, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the binding policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct BindingPolicyEngine {
    pub steps: Vec<Rc<dyn BindingPolicyAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl BindingPolicyEngine {
    pub fn run(&mut self, item: &BindingPolicy) -> f64 {
        let mut current = item.net_value();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn BindingPolicyAdjuster> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[BindingPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_binding_policy_engine(flags: &flags::Registry) -> BindingPolicyEngine {
    BindingPolicyEngine {
        steps: vec![
            Rc::new(CategoryBindingPolicyAdjuster {
                factors: BTreeMap::from([("online".to_string(), 0.91), ("east".to_string(), 1.32)]),
            }) as Rc<dyn BindingPolicyAdjuster>,
            Rc::new(PercentBindingPolicyAdjuster { percent: -14.0 }),
            Rc::new(FlaggedBindingPolicyAdjuster {
                flags: flags.clone(),
                flag: "binding-policy-primary".to_string(),
                percent: -3.9,
            }),
            Rc::new(RoundingBindingPolicyAdjuster { step: 0.5 }),
            Rc::new(MinimumBindingPolicyAdjuster { minimum: 46.3 }),
            Rc::new(TieredBindingPolicyAdjuster {
                bounds: vec![6230.0, 8484.0, 8703.0, 9445.0],
                rates: vec![1.09, 0.91, 0.84, 1.17],
            }),
        ],
        trace: Vec::new(),
        floor: 1.3,
        ceiling: 80177.8,
    }
}
