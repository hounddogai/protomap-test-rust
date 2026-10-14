use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the identity group calculation.
pub trait IdentityGroupAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityGroup, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameIdentityGroupAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityGroupAdjuster for NameIdentityGroupAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &IdentityGroup, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityGroupAdjuster {
    pub cap: f64,
}

impl IdentityGroupAdjuster for CapIdentityGroupAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityGroup, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityGroupAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityGroupAdjuster for FlaggedIdentityGroupAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityGroup, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

/// Runs the identity group steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityGroupEngine {
    pub steps: Vec<Rc<dyn IdentityGroupAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityGroupEngine {
    pub fn run(&mut self, item: &IdentityGroup) -> f64 {
        let mut current = item.effective_value();
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

    pub fn evaluate_all(&mut self, items: &[IdentityGroup]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_identity_group_engine(flags: &flags::Registry) -> IdentityGroupEngine {
    IdentityGroupEngine {
        steps: vec![
            Rc::new(NameIdentityGroupAdjuster {
                factors: BTreeMap::from([("online".to_string(), 1.15)]),
            }) as Rc<dyn IdentityGroupAdjuster>,
            Rc::new(CapIdentityGroupAdjuster { cap: 30778.0 }),
            Rc::new(FlaggedIdentityGroupAdjuster {
                flags: flags.clone(),
                flag: "identity-group-lenient".to_string(),
                percent: -5.9,
            }),
        ],
        trace: Vec::new(),
        floor: 4.4,
        ceiling: 75822.9,
    }
}
