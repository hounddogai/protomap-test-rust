use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the identity override calculation.
pub trait IdentityOverrideModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &IdentityOverride, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapIdentityOverrideAdjuster {
    pub cap: f64,
}

impl IdentityOverrideModifier for CapIdentityOverrideAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &IdentityOverride, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdentityOverrideCreatedAtOffsetAdjuster {
    pub weight: f64,
}

impl IdentityOverrideModifier for IdentityOverrideCreatedAtOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus created at times {}", self.weight)
    }

    fn adjust(&self, item: &IdentityOverride, value: f64) -> f64 {
        value + (item.created_at as f64) * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedIdentityOverrideAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl IdentityOverrideModifier for FlaggedIdentityOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &IdentityOverride, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameIdentityOverrideAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl IdentityOverrideModifier for NameIdentityOverrideAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &IdentityOverride, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentIdentityOverrideAdjuster {
    pub percent: f64,
}

impl IdentityOverrideModifier for PercentIdentityOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &IdentityOverride, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredIdentityOverrideAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl IdentityOverrideModifier for TieredIdentityOverrideAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &IdentityOverride, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the identity override steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct IdentityOverrideEngine {
    pub steps: Vec<Rc<dyn IdentityOverrideModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl IdentityOverrideEngine {
    pub fn run(&mut self, item: &IdentityOverride) -> f64 {
        let mut current = item.net_volume();
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

    pub fn evaluate_all(&mut self, items: &[IdentityOverride]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.run(item));
        }
        results
    }
}

pub fn default_identity_override_engine(flags: &flags::Registry) -> IdentityOverrideEngine {
    IdentityOverrideEngine {
        steps: vec![
            Rc::new(CapIdentityOverrideAdjuster { cap: 5388.0 }) as Rc<dyn IdentityOverrideModifier>,
            Rc::new(IdentityOverrideCreatedAtOffsetAdjuster { weight: 1.33 }),
            Rc::new(FlaggedIdentityOverrideAdjuster {
                flags: flags.clone(),
                flag: "identity-override-default".to_string(),
                percent: 5.9,
            }),
            Rc::new(NameIdentityOverrideAdjuster {
                factors: BTreeMap::from([
                    ("online".to_string(), 0.87),
                    ("central".to_string(), 1.38),
                    ("retail".to_string(), 1.2),
                ]),
            }),
            Rc::new(PercentIdentityOverrideAdjuster { percent: -18.2 }),
            Rc::new(TieredIdentityOverrideAdjuster {
                bounds: vec![429.0, 5194.0, 6668.0, 7081.0],
                rates: vec![0.96, 0.94, 1.07, 0.9],
            }),
        ],
        trace: Vec::new(),
        floor: 4.8,
        ceiling: 77635.1,
    }
}
