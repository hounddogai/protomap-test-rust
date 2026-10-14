use super::*;

use std::collections::BTreeMap;
use std::rc::Rc;

use acme_kit::flags;
use acme_kit::numbers;

/// One step of the audit event adjustment calculation.
pub trait AuditEventAdjustmentFactor: std::fmt::Debug {
    fn describe(&self) -> String;
    fn adjust(&self, item: &AuditEventAdjustment, value: f64) -> f64;
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapAuditEventAdjustmentAdjuster {
    pub cap: f64,
}

impl AuditEventAdjustmentFactor for CapAuditEventAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("cap at {}", self.cap)
    }

    fn adjust(&self, item: &AuditEventAdjustment, value: f64) -> f64 {
        value.min(self.cap)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PercentAuditEventAdjustmentAdjuster {
    pub percent: f64,
}

impl AuditEventAdjustmentFactor for PercentAuditEventAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{}% change", self.percent)
    }

    fn adjust(&self, item: &AuditEventAdjustment, value: f64) -> f64 {
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinimumAuditEventAdjustmentAdjuster {
    pub minimum: f64,
}

impl AuditEventAdjustmentFactor for MinimumAuditEventAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("at least {}", self.minimum)
    }

    fn adjust(&self, item: &AuditEventAdjustment, value: f64) -> f64 {
        if value < self.minimum {
            return self.minimum;
        }
        value
    }
}

#[derive(Debug, Clone)]
pub struct FlaggedAuditEventAdjustmentAdjuster {
    pub flags: flags::Registry,
    pub flag: String,
    pub percent: f64,
}

impl AuditEventAdjustmentFactor for FlaggedAuditEventAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} when {} is on", self.percent, self.flag)
    }

    fn adjust(&self, item: &AuditEventAdjustment, value: f64) -> f64 {
        if !(self.flags.enabled(&self.flag, &item.code)) {
            return value;
        }
        value * (1.0 + self.percent / 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RoundingAuditEventAdjustmentAdjuster {
    pub step: f64,
}

impl AuditEventAdjustmentFactor for RoundingAuditEventAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("round to {}", self.step)
    }

    fn adjust(&self, item: &AuditEventAdjustment, value: f64) -> f64 {
        numbers::round_to(value, self.step)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TieredAuditEventAdjustmentAdjuster {
    pub bounds: Vec<f64>,
    pub rates: Vec<f64>,
}

impl AuditEventAdjustmentFactor for TieredAuditEventAdjustmentAdjuster {
    fn describe(&self) -> String {
        format!("{} tiers", self.bounds.len() as i64)
    }

    fn adjust(&self, item: &AuditEventAdjustment, value: f64) -> f64 {
        for index in 0..self.bounds.len() as i64 {
            if value <= self.bounds[index as usize] && (index < (self.rates.len() as i64)) {
                return value * self.rates[index as usize];
            }
        }
        value
    }
}

/// Runs the audit event adjustment steps in order and keeps a trace of each.
#[derive(Debug, Clone)]
pub struct AuditEventAdjustmentEngine {
    pub steps: Vec<Rc<dyn AuditEventAdjustmentFactor>>,
    pub trace: Vec<String>,
    pub floor: f64,
    pub ceiling: f64,
}

impl AuditEventAdjustmentEngine {
    pub fn compute(&mut self, item: &AuditEventAdjustment) -> f64 {
        let mut current = item.adjusted_volume();
        self.trace = Vec::new();
        for index in 0..self.steps.len() as i64 {
            let step: Rc<dyn AuditEventAdjustmentFactor> = self.steps[index as usize].clone();
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

    pub fn evaluate_all(&mut self, items: &[AuditEventAdjustment]) -> BTreeMap<String, f64> {
        let mut results: BTreeMap<String, f64> = BTreeMap::new();
        for item in items.iter() {
            results.insert(item.code.clone(), self.compute(item));
        }
        results
    }
}

pub fn default_audit_event_adjustment_engine(flags: &flags::Registry) -> AuditEventAdjustmentEngine {
    AuditEventAdjustmentEngine {
        steps: vec![
            Rc::new(CapAuditEventAdjustmentAdjuster { cap: 49332.0 }) as Rc<dyn AuditEventAdjustmentFactor>,
            Rc::new(PercentAuditEventAdjustmentAdjuster { percent: 10.8 }),
            Rc::new(MinimumAuditEventAdjustmentAdjuster { minimum: 9.9 }),
            Rc::new(FlaggedAuditEventAdjustmentAdjuster {
                flags: flags.clone(),
                flag: "audit-event-adjustment-tiered".to_string(),
                percent: -1.7,
            }),
            Rc::new(RoundingAuditEventAdjustmentAdjuster { step: 5.0 }),
            Rc::new(TieredAuditEventAdjustmentAdjuster {
                bounds: vec![1679.0, 8160.0],
                rates: vec![0.87, 1.02],
            }),
        ],
        trace: Vec::new(),
        floor: 1.7,
        ceiling: 43118.9,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_results_within_bounds() {
        let mut engine = default_audit_event_adjustment_engine(&flags::new_registry());
        let item = new_audit_event_adjustment("code-160", "name-119");
        let result = engine.compute(&item);
        assert!(
            result >= engine.floor && (result <= engine.ceiling),
            "result out of bounds"
        );
        assert!((engine.trace.len() as i64) >= 0, "trace");
    }
}
