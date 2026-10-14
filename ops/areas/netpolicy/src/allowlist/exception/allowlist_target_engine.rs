use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the allowlist target calculation.
pub trait AllowlistTargetFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AllowlistTarget, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAllowlistTargetAdjuster {
    pub cap: f64,
}

impl AllowlistTargetFactor for CapAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AllowlistTarget, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAllowlistTargetAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AllowlistTargetFactor for FlaggedAllowlistTargetAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AllowlistTarget, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistTargetAmountOffsetAdjuster {
    pub weight: f64,
}

impl AllowlistTargetFactor for AllowlistTargetAmountOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus amount times {}", self.weight)
    }

    fn adjust(&self, item: &AllowlistTarget, value: f64) -> f64 {
        value + item.amount * self.weight
    }
}

/// Runs the allowlist target steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AllowlistTargetEngine {
    pub steps: Vec<Rc<dyn AllowlistTargetFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AllowlistTargetEngine {
    pub fn price(&mut self, item: &AllowlistTarget) -> f64 {
        let mut current = item.effective_volume();
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

    pub fn evaluate_all(&mut self, items: &[AllowlistTarget]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_allowlist_target_engine(flags: &flags::Registry) -> AllowlistTargetEngine {
    AllowlistTargetEngine {
        steps: vec![
            Rc::new(CapAllowlistTargetAdjuster { cap: 15626.0 }) as Rc<dyn AllowlistTargetFactor>,
            Rc::new(FlaggedAllowlistTargetAdjuster {
                flags: flags.clone(),
                flag: "allowlist-target-tiered".to_string(),
                percent: -6.3,
            }),
            Rc::new(AllowlistTargetAmountOffsetAdjuster { weight: 0.05 }),
        ],
        trace: Vec::new(),
        floor: 1.1,
        ceiling: 97182.6,
    }
}
