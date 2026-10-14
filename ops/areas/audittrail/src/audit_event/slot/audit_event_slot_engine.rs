use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the audit event slot calculation.
pub trait AuditEventSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventSlot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedAuditEventSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AuditEventSlotModifier for FlaggedAuditEventSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AuditEventSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventSlotAdjuster {
    pub step: f64,
}

impl AuditEventSlotModifier for RoundingAuditEventSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventSlotAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventSlotModifier for TieredAuditEventSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventSlot, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the audit event slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventSlotEngine {
    pub steps: Vec<Rc<dyn AuditEventSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventSlotEngine {
    pub fn evaluate(&mut self, item: &AuditEventSlot) -> f64 {
        let mut current = item.projected_exposure();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AuditEventSlotModifier> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.evaluate(item));
        }
        results
    }
}

pub fn default_audit_event_slot_engine(flags: &flags::Registry) -> AuditEventSlotEngine {
    AuditEventSlotEngine {
        steps: vec![
            Rc::new(FlaggedAuditEventSlotAdjuster {
                flags: flags.clone(),
                flag: "audit-event-slot-lenient".to_string(),
                percent: -10.0,
            }) as Rc<dyn AuditEventSlotModifier>,
            Rc::new(RoundingAuditEventSlotAdjuster { step: 0.5 }),
            Rc::new(TieredAuditEventSlotAdjuster {
                bounds: vec![3513.0, 6414.0, 8977.0],
                rates: vec![1.18, 1.16, 1.13],
            }),
        ],
        trace: Vec::new(),
        floor: 4.9,
        ceiling: 82060.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_slot_engine(&flags::new_registry());
        let item = new_audit_event_slot("code-914", "name-192");
        let result = engine.evaluate(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
