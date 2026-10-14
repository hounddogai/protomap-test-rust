use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the pending audit event slot calculation.
pub trait PendingAuditEventSlotModifier: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &PendingAuditEventSlot, value: f64) -> f64;
}

#[derive(Debug, Clone)]
pub struct FlaggedPendingAuditEventSlotAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl PendingAuditEventSlotModifier for FlaggedPendingAuditEventSlotAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &PendingAuditEventSlot, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentPendingAuditEventSlotAdjuster {
    pub percent: f64,
}

impl PendingAuditEventSlotModifier for PercentPendingAuditEventSlotAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &PendingAuditEventSlot, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingPendingAuditEventSlotAdjuster {
    pub step: f64,
}

impl PendingAuditEventSlotModifier for RoundingPendingAuditEventSlotAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &PendingAuditEventSlot, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapPendingAuditEventSlotAdjuster {
    pub cap: f64,
}

impl PendingAuditEventSlotModifier for CapPendingAuditEventSlotAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &PendingAuditEventSlot, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PendingAuditEventSlotBatchSizeOffsetAdjuster {
    pub weight: f64,
}

impl PendingAuditEventSlotModifier for PendingAuditEventSlotBatchSizeOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus batch size times {}", self.weight)
    }

    fn adjust(&self, item: &PendingAuditEventSlot, value: f64) -> f64 {
        value + (item.batch_size as f64) * self.weight
    }
}

/// Runs the pending audit event slot steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct PendingAuditEventSlotEngine {
    pub steps: Vec<Rc<dyn PendingAuditEventSlotModifier>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl PendingAuditEventSlotEngine {
    pub fn compute(&mut self, item: &PendingAuditEventSlot) -> f64 {
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

    pub fn evaluate_all(&mut self, items: &[PendingAuditEventSlot]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_pending_audit_event_slot_engine(flags: &flags::Registry) -> PendingAuditEventSlotEngine {
    PendingAuditEventSlotEngine {
        steps: vec![
            Rc::new(FlaggedPendingAuditEventSlotAdjuster {
                flags: flags.clone(),
                flag: "pending-audit-event-slot-global".to_string(),
                percent: 2.5,
            }) as Rc<dyn PendingAuditEventSlotModifier>,
            Rc::new(PercentPendingAuditEventSlotAdjuster { percent: 17.9 }),
            Rc::new(RoundingPendingAuditEventSlotAdjuster { step: 5.0 }),
            Rc::new(CapPendingAuditEventSlotAdjuster { cap: 40937.0 }),
            Rc::new(PendingAuditEventSlotBatchSizeOffsetAdjuster { weight: 1.61 }),
        ],
        trace: Vec::new(),
        floor: 0.6,
        ceiling: 64830.7,
    }
}
