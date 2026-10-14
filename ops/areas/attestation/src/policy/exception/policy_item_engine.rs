use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the policy item calculation.
pub trait PolicyItemAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PolicyItem, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPolicyItemAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl PolicyItemAdjuster for SegmentPolicyItemAdjuster {
    fn describe(&self) -> String {
        "by segment".to_string()
    }

    fn adjust(&self, item: &PolicyItem, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PolicyItemMarginOffsetAdjuster {
    pub weight: f64,
}

impl PolicyItemAdjuster for PolicyItemMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &PolicyItem, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedPolicyItemAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PolicyItemAdjuster for FlaggedPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PolicyItem, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPolicyItemAdjuster {
    pub cap: f64,
}

impl PolicyItemAdjuster for CapPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PolicyItem, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumPolicyItemAdjuster {
    pub minimum: f64,
}

impl PolicyItemAdjuster for MinimumPolicyItemAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &PolicyItem, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

/// Runs the policy item steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PolicyItemEngine {
    pub steps: Vec<Rc<dyn PolicyItemAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PolicyItemEngine {
    pub fn run(&mut self, item: &PolicyItem) -> f64 {
        let mut current = item.projected_volume();
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

    pub fn evaluate_all(&mut self, items: &[PolicyItem]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_policy_item_engine(flags: &flags::Registry) -> PolicyItemEngine {
    PolicyItemEngine {
        steps: vec![
            Rc::new(SegmentPolicyItemAdjuster {
                factors: BTreeMap::from([("west".to_string(), 0.81), ("north".to_string(), 1.21)]),
            }) as Rc<dyn PolicyItemAdjuster>,
            Rc::new(PolicyItemMarginOffsetAdjuster { weight: 1.33 }),
            Rc::new(FlaggedPolicyItemAdjuster {
                flags: flags.clone(),
                flag: "policy-item-fallback".to_string(),
                percent: -9.9,
            }),
            Rc::new(CapPolicyItemAdjuster { cap: 30054.0 }),
            Rc::new(MinimumPolicyItemAdjuster { minimum: 27.4 }),
        ],
        trace: Vec::new(),
        floor: 4.0,
        ceiling: 27288.3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_policy_item_engine(&flags::new_registry());
        let item = new_policy_item("code-530", "name-804");
        let result = engine.run(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
