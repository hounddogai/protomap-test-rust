use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the scheduled egress policy calculation.
pub trait ScheduledEgressPolicyFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &ScheduledEgressPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentScheduledEgressPolicyAdjuster {
    pub percent: f64,
}

impl ScheduledEgressPolicyFactor for PercentScheduledEgressPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &ScheduledEgressPolicy, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapScheduledEgressPolicyAdjuster {
    pub cap: f64,
}

impl ScheduledEgressPolicyFactor for CapScheduledEgressPolicyAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &ScheduledEgressPolicy, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedScheduledEgressPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl ScheduledEgressPolicyFactor for FlaggedScheduledEgressPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &ScheduledEgressPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClusterScheduledEgressPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl ScheduledEgressPolicyFactor for ClusterScheduledEgressPolicyAdjuster {
    fn describe(&self) -> String {
        "by cluster".to_string()
    }

    fn adjust(&self, item: &ScheduledEgressPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledEgressPolicyAmountOffsetAdjuster {
    pub weight: f64,
}

impl ScheduledEgressPolicyFactor for ScheduledEgressPolicyAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &ScheduledEgressPolicy, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumScheduledEgressPolicyAdjuster {
    pub minimum: f64,
}

impl ScheduledEgressPolicyFactor for MinimumScheduledEgressPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &ScheduledEgressPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the scheduled egress policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct ScheduledEgressPolicyEngine {
    pub steps: Vec<Rc<dyn ScheduledEgressPolicyFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl ScheduledEgressPolicyEngine {
    pub fn run(&mut self, item: &ScheduledEgressPolicy) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[ScheduledEgressPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_scheduled_egress_policy_engine(flags: &flags::Registry) -> ScheduledEgressPolicyEngine {
    ScheduledEgressPolicyEngine {
        steps: vec![
            Rc::new(PercentScheduledEgressPolicyAdjuster { percent: -0.8 }) as Rc<dyn ScheduledEgressPolicyFactor>,
            Rc::new(CapScheduledEgressPolicyAdjuster { cap: 48372.0 }),
            Rc::new(FlaggedScheduledEgressPolicyAdjuster {
                flags: flags.clone(),
                flag: "scheduled-egress-policy-batched".to_string(),
                percent: 4.7,
            }),
            Rc::new(ClusterScheduledEgressPolicyAdjuster {
                factors: BTreeMap::from([
                    ("south".to_string(), 0.81),
                    ("online".to_string(), 0.79),
                    ("north".to_string(), 0.77),
                ]),
            }),
            Rc::new(ScheduledEgressPolicyAmountOffsetAdjuster { weight: 0.21 }),
            Rc::new(MinimumScheduledEgressPolicyAdjuster { minimum: 38.3 }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 18780.8,
    }
}
