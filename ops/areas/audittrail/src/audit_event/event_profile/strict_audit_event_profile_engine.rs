use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the strict audit event profile calculation.
pub trait StrictAuditEventProfileModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &StrictAuditEventProfile, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedStrictAuditEventProfileAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl StrictAuditEventProfileModifier for FlaggedStrictAuditEventProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &StrictAuditEventProfile, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredStrictAuditEventProfileAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl StrictAuditEventProfileModifier for TieredStrictAuditEventProfileAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &StrictAuditEventProfile, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChannelStrictAuditEventProfileAdjuster {
    pub factors: BTreeMap<String, f64>,
}

impl StrictAuditEventProfileModifier for ChannelStrictAuditEventProfileAdjuster {
    fn describe(&self) -> String {
        "by channel".to_string()
    }

    fn adjust(&self, item: &StrictAuditEventProfile, value: f64) -> f64 {
        let Some(factor) = self.factors.get(&item.group_key()).copied() else {
            return value;
        };
        value * factor
    }
}

/// Runs the strict audit event profile steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct StrictAuditEventProfileEngine {
    pub steps: Vec<Rc<dyn StrictAuditEventProfileModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl StrictAuditEventProfileEngine {
    pub fn price(&mut self, item: &StrictAuditEventProfile) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[StrictAuditEventProfile]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_strict_audit_event_profile_engine(flags: &flags::Registry) -> StrictAuditEventProfileEngine {
    StrictAuditEventProfileEngine {
        steps: vec![
            Rc::new(FlaggedStrictAuditEventProfileAdjuster {
                flags: flags.clone(),
                flag: "strict-audit-event-profile-adaptive".to_string(),
                percent: -6.2,
            }) as Rc<dyn StrictAuditEventProfileModifier>,
            Rc::new(TieredStrictAuditEventProfileAdjuster {
                bounds: vec![4009.0, 6017.0, 6505.0, 8319.0],
                rates: vec![1.18, 0.94, 0.83, 0.99],
            }),
            Rc::new(ChannelStrictAuditEventProfileAdjuster {
                factors: BTreeMap::from([
                    ("east".to_string(), 1.08),
                    ("retail".to_string(), 0.73),
                    ("north".to_string(), 0.77),
                ]),
            }),
        ],
        trace: Vec::new(),
        floor: 2.2,
        ceiling: 52709.9,
    }
}
