use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the global redaction group calculation.
pub trait GlobalRedactionGroupFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &GlobalRedactionGroup, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedGlobalRedactionGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl GlobalRedactionGroupFactor for FlaggedGlobalRedactionGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &GlobalRedactionGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapGlobalRedactionGroupAdjuster {
    pub cap: f64,
}

impl GlobalRedactionGroupFactor for CapGlobalRedactionGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &GlobalRedactionGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRedactionGroupRetriesOffsetAdjuster {
    pub weight: f64,
}

impl GlobalRedactionGroupFactor for GlobalRedactionGroupRetriesOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus retries times {}", self.weight)
    }

    fn adjust(&self, item: &GlobalRedactionGroup, value: f64) -> f64 {
        value + (item.retries as f64) * self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LabelGlobalRedactionGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl GlobalRedactionGroupFactor for LabelGlobalRedactionGroupAdjuster {
    fn describe(&self) -> String {
        "by label".to_string()
    }

    fn adjust(&self, item: &GlobalRedactionGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingGlobalRedactionGroupAdjuster {
    pub step: f64,
}

impl GlobalRedactionGroupFactor for RoundingGlobalRedactionGroupAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &GlobalRedactionGroup, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the global redaction group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct GlobalRedactionGroupEngine {
    pub steps: Vec<Rc<dyn GlobalRedactionGroupFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl GlobalRedactionGroupEngine {
    pub fn price(&mut self, item: &GlobalRedactionGroup) -> f64 {
        let mut current = item.adjusted_total();
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

    pub fn evaluate_all(&mut self, items: &[GlobalRedactionGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_global_redaction_group_engine(flags: &flags::Registry) -> GlobalRedactionGroupEngine {
    GlobalRedactionGroupEngine {
        steps: vec![
            Rc::new(FlaggedGlobalRedactionGroupAdjuster {
                flags: flags.clone(),
                flag: "global-redaction-group-default".to_string(),
                percent: -2.1,
            }) as Rc<dyn GlobalRedactionGroupFactor>,
            Rc::new(CapGlobalRedactionGroupAdjuster { cap: 32602.0 }),
            Rc::new(GlobalRedactionGroupRetriesOffsetAdjuster { weight: 0.75 }),
            Rc::new(LabelGlobalRedactionGroupAdjuster {
                factors: BTreeMap::from([
                    ("north".to_string(), 1.02),
                    ("central".to_string(), 0.85),
                    ("online".to_string(), 1.14),
                ]),
            }),
            Rc::new(RoundingGlobalRedactionGroupAdjuster { step: 0.25 }),
        ],
        trace: Vec::new(),
        floor: 0.5,
        ceiling: 50941.9,
    }
}
