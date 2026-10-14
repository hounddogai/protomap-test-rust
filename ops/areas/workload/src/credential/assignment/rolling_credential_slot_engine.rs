use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the rolling credential slot calculation.
pub trait RollingCredentialSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &RollingCredentialSlot, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentRollingCredentialSlotAdjuster {
    pub percent: f64,
}

impl RollingCredentialSlotModifier for PercentRollingCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &RollingCredentialSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapRollingCredentialSlotAdjuster {
    pub cap: f64,
}

impl RollingCredentialSlotModifier for CapRollingCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &RollingCredentialSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameRollingCredentialSlotAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl RollingCredentialSlotModifier for NameRollingCredentialSlotAdjuster {
    fn describe(&self) -> String {
        "by name".to_string()
    }

    fn adjust(&self, item: &RollingCredentialSlot, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumRollingCredentialSlotAdjuster {
    pub minimum: f64,
}

impl RollingCredentialSlotModifier for MinimumRollingCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &RollingCredentialSlot, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedRollingCredentialSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl RollingCredentialSlotModifier for FlaggedRollingCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &RollingCredentialSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingRollingCredentialSlotAdjuster {
    pub step: f64,
}

impl RollingCredentialSlotModifier for RoundingRollingCredentialSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &RollingCredentialSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

/// Runs the rolling credential slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct RollingCredentialSlotEngine {
    pub steps: Vec<Rc<dyn RollingCredentialSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl RollingCredentialSlotEngine {
    pub fn price(&mut self, item: &RollingCredentialSlot) -> f64 {
        let mut current = item.effective_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn RollingCredentialSlotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[RollingCredentialSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_rolling_credential_slot_engine(flags: &flags::Registry) -> RollingCredentialSlotEngine {
    RollingCredentialSlotEngine {
        steps: vec![
            Rc::new(PercentRollingCredentialSlotAdjuster { percent: 7.8 }) as Rc<dyn RollingCredentialSlotModifier>,
            Rc::new(CapRollingCredentialSlotAdjuster { cap: 26910.0 }),
            Rc::new(NameRollingCredentialSlotAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.04),
                    ("south".to_string(), 1.34),
                    ("central".to_string(), 1.2),
                ]),
            }),
            Rc::new(MinimumRollingCredentialSlotAdjuster { minimum: 32.4 }),
            Rc::new(FlaggedRollingCredentialSlotAdjuster {
                flags: flags.clone(),
                flag: "rolling-credential-slot-global".to_string(),
                percent: 4.4,
            }),
            Rc::new(RoundingRollingCredentialSlotAdjuster { step: 1.0 }),
        ],
        trace: Vec::new(),
        floor: 0.2,
        ceiling: 30377.4,
    }
}
