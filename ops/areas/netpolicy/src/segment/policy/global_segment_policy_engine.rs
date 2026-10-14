use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global segment policy calculation.
pub trait GlobalSegmentPolicyModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalSegmentPolicy, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumGlobalSegmentPolicyAdjuster {
    pub minimum: f64,
}

impl GlobalSegmentPolicyModifier for MinimumGlobalSegmentPolicyAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &GlobalSegmentPolicy, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPolicyExpiresAtOffsetAdjuster {
    pub weight: f64,
}

impl GlobalSegmentPolicyModifier for GlobalSegmentPolicyExpiresAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus expires at times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalSegmentPolicy, value: f64) -> f64 {
        value + (item.expires_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalSegmentPolicyAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalSegmentPolicyModifier for FlaggedGlobalSegmentPolicyAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalSegmentPolicy, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalSegmentPolicyAdjuster {
    pub step: f64,
}

impl GlobalSegmentPolicyModifier for RoundingGlobalSegmentPolicyAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalSegmentPolicy, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CategoryGlobalSegmentPolicyAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalSegmentPolicyModifier for CategoryGlobalSegmentPolicyAdjuster {
    fn describe(&self) -> String {
        "by category".to_string()
    }

    fn adjust(&self, item: &GlobalSegmentPolicy, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the global segment policy steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalSegmentPolicyEngine {
    pub steps: Vec<Rc<dyn GlobalSegmentPolicyModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalSegmentPolicyEngine {
    pub fn evaluate(&mut self, item: &GlobalSegmentPolicy) -> f64 {
        let mut current = item.adjusted_volume();
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

    pub fn evaluate_all(&mut self, items: &[GlobalSegmentPolicy]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_global_segment_policy_engine(flags: &flags::Registry) -> GlobalSegmentPolicyEngine {
    GlobalSegmentPolicyEngine {
        steps: vec![
            Rc::new(MinimumGlobalSegmentPolicyAdjuster { minimum: 24.1 }) as Rc<dyn GlobalSegmentPolicyModifier>,
            Rc::new(GlobalSegmentPolicyExpiresAtOffsetAdjuster { weight: 1.17 }),
            Rc::new(FlaggedGlobalSegmentPolicyAdjuster {
                flags: flags.clone(),
                flag: "global-segment-policy-regional".to_string(),
                percent: -9.1,
            }),
            Rc::new(RoundingGlobalSegmentPolicyAdjuster { step: 0.1 }),
            Rc::new(CategoryGlobalSegmentPolicyAdjuster {
                factors: BTreeMap::from([("west".to_string(), 1.19)]),
            }),
        ],
        trace: Vec::new(),
        floor: 4.5,
        ceiling: 47811.2,
    }
}
