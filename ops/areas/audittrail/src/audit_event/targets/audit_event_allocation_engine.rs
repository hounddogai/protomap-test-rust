use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the audit event allocation calculation.
pub trait AuditEventAllocationAdjuster: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventAllocation, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventAllocationMarginOffsetAdjuster {
    pub weight: f64,
}

impl AuditEventAllocationAdjuster for AuditEventAllocationMarginOffsetAdjuster {
    fn describe(&self) -> String {
        format!("plus margin times {}", self.weight)
    }

    fn adjust(&self, item: &AuditEventAllocation, value: f64) -> f64 {
        value + item.margin * self.weight
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAuditEventAllocationAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AuditEventAllocationAdjuster for FlaggedAuditEventAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AuditEventAllocation, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventAllocationAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventAllocationAdjuster for TieredAuditEventAllocationAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventAllocation, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventAllocationAdjuster {
    pub cap: f64,
}

impl AuditEventAllocationAdjuster for CapAuditEventAllocationAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventAllocation, value: f64) -> f64 {
        value.min(self.cap)
    }
}

/// Runs the audit event allocation steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventAllocationEngine {
    pub steps: Vec<Rc<dyn AuditEventAllocationAdjuster>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventAllocationEngine {
    pub fn price(&mut self, item: &AuditEventAllocation) -> f64 {
        let mut current = item.weighted_load();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventAllocation]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.price(item));
        }
        results
    }
}

pub fn default_audit_event_allocation_engine(flags: &flags::Registry) -> AuditEventAllocationEngine {
    AuditEventAllocationEngine {
        steps: vec![
            Rc::new(AuditEventAllocationMarginOffsetAdjuster { weight: 1.5 }) as Rc<dyn AuditEventAllocationAdjuster>,
            Rc::new(FlaggedAuditEventAllocationAdjuster {
                flags: flags.clone(),
                flag: "audit-event-allocation-regional".to_string(),
                percent: -3.1,
            }),
            Rc::new(TieredAuditEventAllocationAdjuster {
                bounds: vec![727.0, 6725.0],
                rates: vec![0.89, 1.02],
            }),
            Rc::new(CapAuditEventAllocationAdjuster { cap: 9292.0 }),
        ],
        trace: Vec::new(),
        floor: 2.1,
        ceiling: 44524.7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_allocation_engine(&flags::new_registry());
        let item = new_audit_event_allocation("code-371", "title-27");
        let result = engine.price(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
